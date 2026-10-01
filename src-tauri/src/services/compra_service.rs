use rusqlite::{params, OptionalExtension};

use crate::db::DbPool;
use crate::error::{AppError, AppResult};
use crate::models::compra::{Compra, CompraConDetalle, CompraDetalleItem, CompraNueva};
use crate::services::audit_service;
use crate::services::stock_service::{self, NuevoMovimiento};

fn map_compra(row: &rusqlite::Row) -> rusqlite::Result<Compra> {
    Ok(Compra {
        id: row.get(0)?,
        proveedor_id: row.get(1)?,
        proveedor_nombre: row.get(2)?,
        numero: row.get(3)?,
        fecha: row.get(4)?,
        subtotal: row.get(5)?,
        impuestos: row.get(6)?,
        total: row.get(7)?,
        estado: row.get(8)?,
        observaciones: row.get(9)?,
    })
}

fn traducir_error(e: rusqlite::Error) -> AppError {
    if let rusqlite::Error::SqliteFailure(_, Some(msg)) = &e {
        if msg.contains("UNIQUE") {
            return AppError::Validacion("Ya existe una compra con ese número.".into());
        }
    }
    AppError::Db(e)
}

pub fn obtener(pool: &DbPool, id: i64) -> AppResult<CompraConDetalle> {
    let conn = pool.get()?;
    let compra = conn
        .query_row(
            "SELECT c.id, c.proveedor_id, pr.razon_social, c.numero, c.fecha, c.subtotal,
                    c.impuestos, c.total, c.estado, c.observaciones
             FROM compras c LEFT JOIN proveedores pr ON pr.id = c.proveedor_id
             WHERE c.id = ?1",
            [id],
            map_compra,
        )
        .optional()?
        .ok_or_else(|| AppError::NoEncontrado("La compra no existe.".into()))?;

    let mut stmt = conn.prepare(
        "SELECT cd.id, cd.producto_id, p.nombre, cd.cantidad, cd.costo_unitario, cd.subtotal
         FROM compra_detalle cd JOIN productos p ON p.id = cd.producto_id
         WHERE cd.compra_id = ?1 ORDER BY cd.id",
    )?;
    let rows = stmt.query_map([id], |r| {
        Ok(CompraDetalleItem {
            id: r.get(0)?,
            producto_id: r.get(1)?,
            producto_nombre: r.get(2)?,
            cantidad: r.get(3)?,
            costo_unitario: r.get(4)?,
            subtotal: r.get(5)?,
        })
    })?;
    let mut items = Vec::new();
    for r in rows {
        items.push(r?);
    }

    Ok(CompraConDetalle { compra, items })
}

pub fn listar(pool: &DbPool, proveedor_id: Option<i64>) -> AppResult<Vec<Compra>> {
    let conn = pool.get()?;
    let sql = "SELECT c.id, c.proveedor_id, pr.razon_social, c.numero, c.fecha, c.subtotal,
                      c.impuestos, c.total, c.estado, c.observaciones
               FROM compras c LEFT JOIN proveedores pr ON pr.id = c.proveedor_id
               WHERE (?1 IS NULL OR c.proveedor_id = ?1)
               ORDER BY c.fecha DESC";
    let mut stmt = conn.prepare(sql)?;
    let rows = stmt.query_map([proveedor_id], map_compra)?;
    let mut out = Vec::new();
    for r in rows {
        out.push(r?);
    }
    Ok(out)
}

pub fn crear(pool: &DbPool, datos: CompraNueva) -> AppResult<CompraConDetalle> {
    if datos.numero.trim().is_empty() {
        return Err(AppError::Validacion(
            "El número de compra es obligatorio.".into(),
        ));
    }
    if datos.items.is_empty() {
        return Err(AppError::Validacion(
            "La compra debe tener al menos un producto.".into(),
        ));
    }
    for item in &datos.items {
        if item.cantidad <= 0 {
            return Err(AppError::Validacion(
                "La cantidad de cada ítem debe ser mayor que cero.".into(),
            ));
        }
        if item.costo_unitario < 0 {
            return Err(AppError::Validacion(
                "El costo unitario no puede ser negativo.".into(),
            ));
        }
    }

    let subtotal: i64 = datos
        .items
        .iter()
        .map(|i| i.cantidad * i.costo_unitario)
        .sum();

    let mut conn = pool.get()?;
    let tx = conn.transaction()?;

    tx.execute(
        "INSERT INTO compras (proveedor_id, numero, subtotal, impuestos, total, estado, observaciones)
         VALUES (?1, ?2, ?3, 0, ?3, 'registrada', ?4)",
        params![datos.proveedor_id, datos.numero, subtotal, datos.observaciones],
    )
    .map_err(traducir_error)?;
    let compra_id = tx.last_insert_rowid();

    for item in &datos.items {
        let item_subtotal = item.cantidad * item.costo_unitario;
        tx.execute(
            "INSERT INTO compra_detalle (compra_id, producto_id, cantidad, costo_unitario, subtotal)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![compra_id, item.producto_id, item.cantidad, item.costo_unitario, item_subtotal],
        )?;

        stock_service::aplicar_movimiento_tx(
            &tx,
            NuevoMovimiento {
                producto_id: item.producto_id,
                tipo: "entrada".into(),
                cantidad: item.cantidad,
                costo_unitario: Some(item.costo_unitario),
                motivo: None,
                referencia_tipo: Some("compra".into()),
                referencia_id: Some(compra_id),
            },
        )?;
    }

    tx.commit()?;
    drop(conn);

    let creada = obtener(pool, compra_id)?;
    audit_service::registrar(pool, "compra", Some(compra_id), "crear", &creada.compra)?;
    Ok(creada)
}

pub fn anular(pool: &DbPool, id: i64, motivo: String) -> AppResult<CompraConDetalle> {
    if motivo.trim().is_empty() {
        return Err(AppError::Validacion(
            "El motivo de anulación es obligatorio.".into(),
        ));
    }
    let existente = obtener(pool, id)?;
    if existente.compra.estado == "anulada" {
        return Err(AppError::Validacion("La compra ya está anulada.".into()));
    }

    let mut conn = pool.get()?;
    let tx = conn.transaction()?;

    for item in &existente.items {
        stock_service::aplicar_movimiento_tx(
            &tx,
            NuevoMovimiento {
                producto_id: item.producto_id,
                tipo: "salida".into(),
                cantidad: item.cantidad,
                costo_unitario: None,
                motivo: Some(format!(
                    "Anulación de compra {}: {motivo}",
                    existente.compra.numero
                )),
                referencia_tipo: Some("anulacion_compra".into()),
                referencia_id: Some(id),
            },
        )?;
    }

    tx.execute(
        "UPDATE compras SET estado = 'anulada',
            observaciones = COALESCE(observaciones || ' | ', '') || ?1
         WHERE id = ?2",
        params![format!("Anulada: {motivo}"), id],
    )?;
    tx.commit()?;
    drop(conn);

    let actualizada = obtener(pool, id)?;
    audit_service::registrar(pool, "compra", Some(id), "anular", &actualizada.compra)?;
    Ok(actualizada)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::init_pool;
    use crate::models::compra::CompraItemNuevo;
    use tempfile::tempdir;

    fn pool_de_prueba() -> DbPool {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.sqlite3");
        let pool = init_pool(&path).unwrap();
        std::mem::forget(dir);
        pool
    }

    fn crear_producto(pool: &DbPool, sku: &str) -> i64 {
        let conn = pool.get().unwrap();
        conn.execute(
            "INSERT INTO productos (sku, nombre, unidad_medida, precio_costo, precio_venta)
             VALUES (?1, 'Producto de prueba', 'unidad', 0, 0)",
            params![sku],
        )
        .unwrap();
        conn.last_insert_rowid()
    }

    #[test]
    fn crear_compra_aumenta_stock_del_producto() {
        let pool = pool_de_prueba();
        let producto_id = crear_producto(&pool, "SKU-COMPRA-1");

        let compra = crear(
            &pool,
            CompraNueva {
                proveedor_id: None,
                numero: "C-001".into(),
                observaciones: None,
                items: vec![CompraItemNuevo {
                    producto_id,
                    cantidad: 20,
                    costo_unitario: 500,
                }],
            },
        )
        .unwrap();

        assert_eq!(compra.compra.subtotal, 10_000);
        assert_eq!(compra.compra.total, 10_000);

        let conn = pool.get().unwrap();
        let stock: i64 = conn
            .query_row(
                "SELECT stock_actual FROM productos WHERE id = ?1",
                [producto_id],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(stock, 20);
    }

    #[test]
    fn anular_compra_revierte_el_stock() {
        let pool = pool_de_prueba();
        let producto_id = crear_producto(&pool, "SKU-COMPRA-2");

        let compra = crear(
            &pool,
            CompraNueva {
                proveedor_id: None,
                numero: "C-002".into(),
                observaciones: None,
                items: vec![CompraItemNuevo {
                    producto_id,
                    cantidad: 15,
                    costo_unitario: 500,
                }],
            },
        )
        .unwrap();

        anular(&pool, compra.compra.id, "Error de digitación".into()).unwrap();

        let stock: i64 = {
            let conn = pool.get().unwrap();
            conn.query_row(
                "SELECT stock_actual FROM productos WHERE id = ?1",
                [producto_id],
                |r| r.get(0),
            )
            .unwrap()
        };
        assert_eq!(stock, 0);

        let compra_actualizada = obtener(&pool, compra.compra.id).unwrap();
        assert_eq!(compra_actualizada.compra.estado, "anulada");
    }

    #[test]
    fn no_se_puede_anular_dos_veces() {
        let pool = pool_de_prueba();
        let producto_id = crear_producto(&pool, "SKU-COMPRA-3");
        let compra = crear(
            &pool,
            CompraNueva {
                proveedor_id: None,
                numero: "C-003".into(),
                observaciones: None,
                items: vec![CompraItemNuevo {
                    producto_id,
                    cantidad: 5,
                    costo_unitario: 100,
                }],
            },
        )
        .unwrap();
        anular(&pool, compra.compra.id, "motivo".into()).unwrap();
        let resultado = anular(&pool, compra.compra.id, "motivo".into());
        assert!(resultado.is_err());
    }
}

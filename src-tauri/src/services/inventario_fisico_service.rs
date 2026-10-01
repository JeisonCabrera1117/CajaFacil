use rusqlite::{params, OptionalExtension};

use crate::db::DbPool;
use crate::error::{AppError, AppResult};
use crate::models::inventario_fisico::{
    InventarioFisico, InventarioFisicoConDetalle, InventarioFisicoDetalle, RegistrarConteo,
};
use crate::services::audit_service;
use crate::services::stock_service::{self, NuevoMovimiento};

fn map_cabecera(row: &rusqlite::Row) -> rusqlite::Result<InventarioFisico> {
    Ok(InventarioFisico {
        id: row.get(0)?,
        fecha: row.get(1)?,
        estado: row.get(2)?,
        observaciones: row.get(3)?,
    })
}

fn obtener_cabecera(pool: &DbPool, id: i64) -> AppResult<InventarioFisico> {
    let conn = pool.get()?;
    conn.query_row(
        "SELECT id, fecha, estado, observaciones FROM inventario_fisico WHERE id = ?1",
        [id],
        map_cabecera,
    )
    .optional()?
    .ok_or_else(|| AppError::NoEncontrado("La toma de inventario no existe.".into()))
}

pub fn iniciar(pool: &DbPool, observaciones: Option<String>) -> AppResult<InventarioFisico> {
    let conn = pool.get()?;
    conn.execute(
        "INSERT INTO inventario_fisico (estado, observaciones) VALUES ('abierto', ?1)",
        params![observaciones],
    )?;
    let id = conn.last_insert_rowid();
    drop(conn);
    let creado = obtener_cabecera(pool, id)?;
    audit_service::registrar(pool, "inventario_fisico", Some(id), "crear", &creado)?;
    Ok(creado)
}

pub fn listar(pool: &DbPool) -> AppResult<Vec<InventarioFisico>> {
    let conn = pool.get()?;
    let mut stmt = conn.prepare(
        "SELECT id, fecha, estado, observaciones FROM inventario_fisico ORDER BY fecha DESC",
    )?;
    let rows = stmt.query_map([], map_cabecera)?;
    let mut out = Vec::new();
    for r in rows {
        out.push(r?);
    }
    Ok(out)
}

pub fn obtener(pool: &DbPool, id: i64) -> AppResult<InventarioFisicoConDetalle> {
    let cabecera = obtener_cabecera(pool, id)?;
    let conn = pool.get()?;
    let mut stmt = conn.prepare(
        "SELECT d.id, d.producto_id, p.nombre, p.sku, d.stock_sistema, d.stock_contado, d.diferencia
         FROM inventario_fisico_detalle d JOIN productos p ON p.id = d.producto_id
         WHERE d.inventario_fisico_id = ?1 ORDER BY p.nombre",
    )?;
    let rows = stmt.query_map([id], |r| {
        Ok(InventarioFisicoDetalle {
            id: r.get(0)?,
            producto_id: r.get(1)?,
            producto_nombre: r.get(2)?,
            sku: r.get(3)?,
            stock_sistema: r.get(4)?,
            stock_contado: r.get(5)?,
            diferencia: r.get(6)?,
        })
    })?;
    let mut detalle = Vec::new();
    for r in rows {
        detalle.push(r?);
    }
    Ok(InventarioFisicoConDetalle { cabecera, detalle })
}

pub fn registrar_conteo(
    pool: &DbPool,
    datos: RegistrarConteo,
) -> AppResult<InventarioFisicoConDetalle> {
    let cabecera = obtener_cabecera(pool, datos.inventario_fisico_id)?;
    if cabecera.estado != "abierto" {
        return Err(AppError::Validacion(
            "Esta toma de inventario ya está cerrada.".into(),
        ));
    }
    if datos.stock_contado < 0 {
        return Err(AppError::Validacion(
            "El stock contado no puede ser negativo.".into(),
        ));
    }

    let conn = pool.get()?;
    let stock_sistema: i64 = conn
        .query_row(
            "SELECT stock_actual FROM productos WHERE id = ?1",
            [datos.producto_id],
            |r| r.get(0),
        )
        .map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => {
                AppError::NoEncontrado("El producto no existe.".into())
            }
            other => AppError::Db(other),
        })?;
    let diferencia = datos.stock_contado - stock_sistema;

    let existente: Option<i64> = conn
        .query_row(
            "SELECT id FROM inventario_fisico_detalle WHERE inventario_fisico_id = ?1 AND producto_id = ?2",
            params![datos.inventario_fisico_id, datos.producto_id],
            |r| r.get(0),
        )
        .optional()?;

    match existente {
        Some(detalle_id) => {
            conn.execute(
                "UPDATE inventario_fisico_detalle
                    SET stock_sistema = ?1, stock_contado = ?2, diferencia = ?3
                 WHERE id = ?4",
                params![stock_sistema, datos.stock_contado, diferencia, detalle_id],
            )?;
        }
        None => {
            conn.execute(
                "INSERT INTO inventario_fisico_detalle
                    (inventario_fisico_id, producto_id, stock_sistema, stock_contado, diferencia)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                params![
                    datos.inventario_fisico_id,
                    datos.producto_id,
                    stock_sistema,
                    datos.stock_contado,
                    diferencia
                ],
            )?;
        }
    }
    drop(conn);
    obtener(pool, datos.inventario_fisico_id)
}

pub fn cerrar(pool: &DbPool, id: i64) -> AppResult<InventarioFisicoConDetalle> {
    let actual = obtener(pool, id)?;
    if actual.cabecera.estado != "abierto" {
        return Err(AppError::Validacion(
            "Esta toma de inventario ya está cerrada.".into(),
        ));
    }

    let mut conn = pool.get()?;
    let tx = conn.transaction()?;

    for detalle in actual.detalle.iter().filter(|d| d.diferencia != 0) {
        stock_service::aplicar_movimiento_tx(
            &tx,
            NuevoMovimiento {
                producto_id: detalle.producto_id,
                tipo: "ajuste".into(),
                cantidad: detalle.diferencia,
                costo_unitario: None,
                motivo: Some(format!("Ajuste por toma de inventario física #{id}")),
                referencia_tipo: Some("inventario_fisico".into()),
                referencia_id: Some(id),
            },
        )?;
    }

    tx.execute(
        "UPDATE inventario_fisico SET estado = 'cerrado' WHERE id = ?1",
        [id],
    )?;
    tx.commit()?;
    drop(conn);

    let cerrada = obtener(pool, id)?;
    audit_service::registrar(
        pool,
        "inventario_fisico",
        Some(id),
        "modificar",
        &cerrada.cabecera,
    )?;
    Ok(cerrada)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::init_pool;
    use tempfile::tempdir;

    fn pool_de_prueba() -> DbPool {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.sqlite3");
        let pool = init_pool(&path).unwrap();
        std::mem::forget(dir);
        pool
    }

    fn crear_producto_con_stock(pool: &DbPool, sku: &str, stock: i64) -> i64 {
        let conn = pool.get().unwrap();
        conn.execute(
            "INSERT INTO productos (sku, nombre, unidad_medida, precio_costo, precio_venta, stock_actual)
             VALUES (?1, 'Producto de prueba', 'unidad', 100, 200, ?2)",
            params![sku, stock],
        )
        .unwrap();
        conn.last_insert_rowid()
    }

    #[test]
    fn cerrar_ajusta_stock_segun_diferencias() {
        let pool = pool_de_prueba();
        let producto_id = crear_producto_con_stock(&pool, "SKU-INV-1", 10);

        let cabecera = iniciar(&pool, None).unwrap();
        registrar_conteo(
            &pool,
            RegistrarConteo {
                inventario_fisico_id: cabecera.id,
                producto_id,
                stock_contado: 7,
            },
        )
        .unwrap();

        let cerrada = cerrar(&pool, cabecera.id).unwrap();
        assert_eq!(cerrada.cabecera.estado, "cerrado");
        assert_eq!(cerrada.detalle[0].diferencia, -3);

        let conn = pool.get().unwrap();
        let stock: i64 = conn
            .query_row(
                "SELECT stock_actual FROM productos WHERE id = ?1",
                [producto_id],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(stock, 7);
    }

    #[test]
    fn no_se_puede_cerrar_dos_veces() {
        let pool = pool_de_prueba();
        let producto_id = crear_producto_con_stock(&pool, "SKU-INV-2", 5);
        let cabecera = iniciar(&pool, None).unwrap();
        registrar_conteo(
            &pool,
            RegistrarConteo {
                inventario_fisico_id: cabecera.id,
                producto_id,
                stock_contado: 5,
            },
        )
        .unwrap();
        cerrar(&pool, cabecera.id).unwrap();
        assert!(cerrar(&pool, cabecera.id).is_err());
    }
}

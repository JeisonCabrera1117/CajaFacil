use rusqlite::{params, OptionalExtension};

use crate::db::DbPool;
use crate::error::{AppError, AppResult};
use crate::models::devolucion::{Devolucion, DevolucionDetalleItem, DevolucionNueva};
use crate::services::audit_service;
use crate::services::stock_service::{self, NuevoMovimiento};
use crate::services::venta_service;

fn obtener(pool: &DbPool, id: i64) -> AppResult<Devolucion> {
    let conn = pool.get()?;
    let (venta_id, fecha, motivo, total_devuelto): (i64, String, String, i64) = conn
        .query_row(
            "SELECT venta_id, fecha, motivo, total_devuelto FROM devoluciones WHERE id = ?1",
            [id],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
        )
        .optional()?
        .ok_or_else(|| AppError::NoEncontrado("La devolución no existe.".into()))?;

    let mut stmt = conn.prepare(
        "SELECT dd.id, dd.venta_detalle_id, p.nombre, dd.cantidad, dd.valor
         FROM devolucion_detalle dd
         JOIN venta_detalle vd ON vd.id = dd.venta_detalle_id
         JOIN productos p ON p.id = vd.producto_id
         WHERE dd.devolucion_id = ?1 ORDER BY dd.id",
    )?;
    let rows = stmt.query_map([id], |r| {
        Ok(DevolucionDetalleItem {
            id: r.get(0)?,
            venta_detalle_id: r.get(1)?,
            producto_nombre: r.get(2)?,
            cantidad: r.get(3)?,
            valor: r.get(4)?,
        })
    })?;
    let mut items = Vec::new();
    for r in rows {
        items.push(r?);
    }

    Ok(Devolucion {
        id,
        venta_id,
        fecha,
        motivo,
        total_devuelto,
        items,
    })
}

pub fn crear(pool: &DbPool, datos: DevolucionNueva) -> AppResult<Devolucion> {
    if datos.motivo.trim().is_empty() {
        return Err(AppError::Validacion(
            "El motivo de la devolución es obligatorio.".into(),
        ));
    }
    if datos.items.is_empty() {
        return Err(AppError::Validacion(
            "Debe indicar al menos un producto a devolver.".into(),
        ));
    }

    let venta = venta_service::obtener(pool, datos.venta_id)?;
    if venta.venta.estado != "completada" {
        return Err(AppError::Validacion(
            "Solo se pueden devolver productos de una venta completada.".into(),
        ));
    }

    struct Linea {
        producto_id: i64,
        venta_detalle_id: i64,
        cantidad: i64,
        valor: i64,
    }
    let mut lineas = Vec::with_capacity(datos.items.len());
    let mut total_devuelto = 0i64;

    for item in &datos.items {
        let detalle = venta
            .items
            .iter()
            .find(|d| d.id == item.venta_detalle_id)
            .ok_or_else(|| {
                AppError::Validacion(
                    "El detalle de venta indicado no pertenece a esta venta.".into(),
                )
            })?;
        let restante = detalle.cantidad - detalle.cantidad_devuelta;
        if item.cantidad <= 0 || item.cantidad > restante {
            return Err(AppError::Validacion(format!(
                "No se puede devolver esa cantidad de \"{}\": quedan {} unidades disponibles.",
                detalle.producto_nombre, restante
            )));
        }
        // Aproximación: valor unitario neto promedio de la línea (subtotal / cantidad).
        let valor_unitario = detalle.subtotal / detalle.cantidad;
        let valor = valor_unitario * item.cantidad;
        total_devuelto += valor;
        lineas.push(Linea {
            producto_id: detalle.producto_id,
            venta_detalle_id: item.venta_detalle_id,
            cantidad: item.cantidad,
            valor,
        });
    }

    let mut conn = pool.get()?;
    let tx = conn.transaction()?;

    tx.execute(
        "INSERT INTO devoluciones (venta_id, motivo, total_devuelto) VALUES (?1, ?2, ?3)",
        params![datos.venta_id, datos.motivo, total_devuelto],
    )?;
    let devolucion_id = tx.last_insert_rowid();

    for linea in &lineas {
        tx.execute(
            "INSERT INTO devolucion_detalle (devolucion_id, venta_detalle_id, cantidad, valor)
             VALUES (?1,?2,?3,?4)",
            params![
                devolucion_id,
                linea.venta_detalle_id,
                linea.cantidad,
                linea.valor
            ],
        )?;

        stock_service::aplicar_movimiento_tx(
            &tx,
            NuevoMovimiento {
                producto_id: linea.producto_id,
                tipo: "devolucion".into(),
                cantidad: linea.cantidad,
                costo_unitario: None,
                motivo: None,
                referencia_tipo: Some("devolucion".into()),
                referencia_id: Some(devolucion_id),
            },
        )?;
    }

    tx.commit()?;
    drop(conn);

    let creada = obtener(pool, devolucion_id)?;
    audit_service::registrar(pool, "devolucion", Some(devolucion_id), "crear", &creada)?;
    Ok(creada)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::devolucion::DevolucionItemNuevo;
    use crate::models::venta::{VentaConDetalle, VentaItemNuevo, VentaNueva};
    use crate::services::devolucion_service;
    use tempfile::tempdir;

    fn pool_de_prueba() -> DbPool {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.sqlite3");
        let pool = crate::db::init_pool(&path).unwrap();
        std::mem::forget(dir);
        pool
    }

    fn crear_producto_con_stock(pool: &DbPool, sku: &str, stock: i64) -> i64 {
        let conn = pool.get().unwrap();
        conn.execute(
            "INSERT INTO productos (sku, nombre, unidad_medida, precio_costo, precio_venta, stock_actual)
             VALUES (?1, 'Producto de prueba', 'unidad', 100, 5000, ?2)",
            params![sku, stock],
        )
        .unwrap();
        conn.last_insert_rowid()
    }

    fn crear_venta_simple(pool: &DbPool, producto_id: i64, cantidad: i64) -> VentaConDetalle {
        venta_service::crear(
            pool,
            VentaNueva {
                cliente_id: None,
                metodo_pago: "tarjeta".into(),
                valor_recibido: None,
                descuento_global: 0,
                items: vec![VentaItemNuevo {
                    producto_id,
                    cantidad,
                    precio_unitario: 5000,
                    descuento_pct: 0.0,
                    descuento_valor: 0,
                    impuesto_pct: 0.0,
                }],
            },
        )
        .unwrap()
    }

    #[test]
    fn devolucion_repone_stock() {
        let pool = pool_de_prueba();
        let producto_id = crear_producto_con_stock(&pool, "SKU-D1", 10);
        let venta = crear_venta_simple(&pool, producto_id, 4);
        let detalle_id = venta.items[0].id;

        let devolucion = crear(
            &pool,
            DevolucionNueva {
                venta_id: venta.venta.id,
                motivo: "Producto defectuoso".into(),
                items: vec![DevolucionItemNuevo {
                    venta_detalle_id: detalle_id,
                    cantidad: 2,
                }],
            },
        )
        .unwrap();

        assert_eq!(devolucion.total_devuelto, 10_000);

        let conn = pool.get().unwrap();
        let stock: i64 = conn
            .query_row(
                "SELECT stock_actual FROM productos WHERE id = ?1",
                [producto_id],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(stock, 8); // 10 - 4 (venta) + 2 (devolución)
    }

    #[test]
    fn no_permite_devolver_mas_de_lo_vendido() {
        let pool = pool_de_prueba();
        let producto_id = crear_producto_con_stock(&pool, "SKU-D2", 10);
        let venta = crear_venta_simple(&pool, producto_id, 3);
        let detalle_id = venta.items[0].id;

        let resultado = crear(
            &pool,
            DevolucionNueva {
                venta_id: venta.venta.id,
                motivo: "motivo".into(),
                items: vec![DevolucionItemNuevo {
                    venta_detalle_id: detalle_id,
                    cantidad: 4,
                }],
            },
        );
        assert!(resultado.is_err());
    }

    #[test]
    fn no_permite_devolver_dos_veces_la_misma_cantidad() {
        let pool = pool_de_prueba();
        let producto_id = crear_producto_con_stock(&pool, "SKU-D3", 10);
        let venta = crear_venta_simple(&pool, producto_id, 3);
        let detalle_id = venta.items[0].id;

        crear(
            &pool,
            DevolucionNueva {
                venta_id: venta.venta.id,
                motivo: "motivo".into(),
                items: vec![DevolucionItemNuevo {
                    venta_detalle_id: detalle_id,
                    cantidad: 2,
                }],
            },
        )
        .unwrap();

        let resultado = crear(
            &pool,
            DevolucionNueva {
                venta_id: venta.venta.id,
                motivo: "motivo".into(),
                items: vec![DevolucionItemNuevo {
                    venta_detalle_id: detalle_id,
                    cantidad: 2,
                }],
            },
        );
        assert!(resultado.is_err());
    }

    #[test]
    fn venta_con_devolucion_no_se_puede_anular() {
        let pool = pool_de_prueba();
        let producto_id = crear_producto_con_stock(&pool, "SKU-D4", 10);
        let venta = crear_venta_simple(&pool, producto_id, 3);
        let detalle_id = venta.items[0].id;

        devolucion_service::crear(
            &pool,
            DevolucionNueva {
                venta_id: venta.venta.id,
                motivo: "motivo".into(),
                items: vec![DevolucionItemNuevo {
                    venta_detalle_id: detalle_id,
                    cantidad: 1,
                }],
            },
        )
        .unwrap();

        assert!(venta_service::anular(&pool, venta.venta.id, "motivo".into()).is_err());
    }
}

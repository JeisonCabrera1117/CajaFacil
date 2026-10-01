use rusqlite::{params, OptionalExtension};

use crate::db::DbPool;
use crate::error::{AppError, AppResult};
use crate::models::caja::CajaSesion;
use crate::services::audit_service;

const COLUMNAS: &str = "id, fecha_apertura, fecha_cierre, monto_apertura, monto_cierre_sistema,
    monto_cierre_contado, diferencia, observaciones";

fn map(row: &rusqlite::Row) -> rusqlite::Result<CajaSesion> {
    Ok(CajaSesion {
        id: row.get(0)?,
        fecha_apertura: row.get(1)?,
        fecha_cierre: row.get(2)?,
        monto_apertura: row.get(3)?,
        monto_cierre_sistema: row.get(4)?,
        monto_cierre_contado: row.get(5)?,
        diferencia: row.get(6)?,
        observaciones: row.get(7)?,
    })
}

pub fn estado_actual(pool: &DbPool) -> AppResult<Option<CajaSesion>> {
    let conn = pool.get()?;
    let sql = format!("SELECT {COLUMNAS} FROM caja_sesiones WHERE fecha_cierre IS NULL");
    conn.query_row(&sql, [], map)
        .optional()
        .map_err(AppError::from)
}

fn obtener(pool: &DbPool, id: i64) -> AppResult<CajaSesion> {
    let conn = pool.get()?;
    let sql = format!("SELECT {COLUMNAS} FROM caja_sesiones WHERE id = ?1");
    conn.query_row(&sql, [id], map)
        .optional()?
        .ok_or_else(|| AppError::NoEncontrado("La sesión de caja no existe.".into()))
}

pub fn abrir(pool: &DbPool, monto_apertura: i64) -> AppResult<CajaSesion> {
    if monto_apertura < 0 {
        return Err(AppError::Validacion(
            "El monto de apertura no puede ser negativo.".into(),
        ));
    }
    if estado_actual(pool)?.is_some() {
        return Err(AppError::Validacion(
            "Ya hay una caja abierta. Ciérrela antes de abrir una nueva.".into(),
        ));
    }
    let conn = pool.get()?;
    conn.execute(
        "INSERT INTO caja_sesiones (monto_apertura) VALUES (?1)",
        params![monto_apertura],
    )?;
    let id = conn.last_insert_rowid();
    drop(conn);
    let creada = obtener(pool, id)?;
    audit_service::registrar(pool, "caja_sesion", Some(id), "crear", &creada)?;
    Ok(creada)
}

pub fn cerrar(
    pool: &DbPool,
    monto_cierre_contado: i64,
    observaciones: Option<String>,
) -> AppResult<CajaSesion> {
    if monto_cierre_contado < 0 {
        return Err(AppError::Validacion(
            "El monto contado no puede ser negativo.".into(),
        ));
    }
    let actual = estado_actual(pool)?
        .ok_or_else(|| AppError::Validacion("No hay una caja abierta.".into()))?;

    let conn = pool.get()?;
    // Efectivo neto de ventas completadas registradas en esta sesión de caja.
    let neto_efectivo: i64 = conn.query_row(
        "SELECT COALESCE(SUM(valor_recibido - cambio), 0) FROM ventas
         WHERE caja_sesion_id = ?1 AND estado = 'completada' AND metodo_pago = 'efectivo'",
        [actual.id],
        |r| r.get(0),
    )?;
    let monto_cierre_sistema = actual.monto_apertura + neto_efectivo;
    let diferencia = monto_cierre_contado - monto_cierre_sistema;

    conn.execute(
        "UPDATE caja_sesiones SET fecha_cierre = datetime('now'), monto_cierre_sistema = ?1,
            monto_cierre_contado = ?2, diferencia = ?3, observaciones = ?4
         WHERE id = ?5",
        params![
            monto_cierre_sistema,
            monto_cierre_contado,
            diferencia,
            observaciones,
            actual.id
        ],
    )?;
    drop(conn);
    let cerrada = obtener(pool, actual.id)?;
    audit_service::registrar(pool, "caja_sesion", Some(actual.id), "modificar", &cerrada)?;
    Ok(cerrada)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::venta::{VentaItemNuevo, VentaNueva};
    use crate::services::venta_service;
    use tempfile::tempdir;

    fn pool_de_prueba() -> DbPool {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.sqlite3");
        let pool = crate::db::init_pool(&path).unwrap();
        std::mem::forget(dir);
        pool
    }

    #[test]
    fn no_permite_abrir_dos_cajas() {
        let pool = pool_de_prueba();
        abrir(&pool, 50_000).unwrap();
        assert!(abrir(&pool, 50_000).is_err());
    }

    #[test]
    fn no_permite_cerrar_sin_caja_abierta() {
        let pool = pool_de_prueba();
        assert!(cerrar(&pool, 0, None).is_err());
    }

    #[test]
    fn cierre_calcula_diferencia_con_ventas_en_efectivo() {
        let pool = pool_de_prueba();
        {
            let conn = pool.get().unwrap();
            conn.execute(
                "UPDATE empresa_config SET arqueo_activo = 1 WHERE id = 1",
                [],
            )
            .unwrap();
            conn.execute(
                "INSERT INTO productos (sku, nombre, unidad_medida, precio_costo, precio_venta, stock_actual)
                 VALUES ('SKU-CAJA-1', 'Producto', 'unidad', 100, 5000, 10)",
                [],
            )
            .unwrap();
        }
        let producto_id: i64 = pool
            .get()
            .unwrap()
            .query_row(
                "SELECT id FROM productos WHERE sku = 'SKU-CAJA-1'",
                [],
                |r| r.get(0),
            )
            .unwrap();

        let sesion = abrir(&pool, 100_000).unwrap();

        venta_service::crear(
            &pool,
            VentaNueva {
                cliente_id: None,
                metodo_pago: "efectivo".into(),
                valor_recibido: Some(10_000),
                descuento_global: 0,
                items: vec![VentaItemNuevo {
                    producto_id,
                    cantidad: 1,
                    precio_unitario: 5000,
                    descuento_pct: 0.0,
                    descuento_valor: 0,
                    impuesto_pct: 0.0,
                }],
            },
        )
        .unwrap();

        // Efectivo neto de esta venta: 10_000 recibidos - 5_000 de cambio = 5_000.
        let cerrada = cerrar(&pool, 105_000, None).unwrap();
        assert_eq!(cerrada.monto_cierre_sistema, Some(105_000));
        assert_eq!(cerrada.diferencia, Some(0));
        assert_eq!(cerrada.id, sesion.id);
    }
}

use rusqlite::params;

use crate::db::DbPool;
use crate::error::{AppError, AppResult};
use crate::models::dashboard::{
    Comparativo, DashboardGraficas, DashboardIndicadores, ProductoCantidad, PuntoSerie,
};

// `fecha` se guarda con `datetime('now')` (UTC puro). Toda comparación por día
// calendario tiene que pasar por `'localtime'` en AMBOS lados (la columna y
// cualquier `'now'`), o el corte de "hoy"/"este mes" queda a la hora UTC en
// vez de la hora real del usuario — a UTC-5 eso corre el borde del día 5
// horas, y "ventas de hoy" empieza a devolver $0 desde las 7pm en adelante.

pub fn indicadores(pool: &DbPool) -> AppResult<DashboardIndicadores> {
    let conn = pool.get()?;

    let ventas_hoy: i64 = conn.query_row(
        "SELECT COALESCE(SUM(total),0) FROM ventas
         WHERE estado='completada' AND date(fecha,'localtime') = date('now','localtime')",
        [],
        |r| r.get(0),
    )?;
    let ventas_semana: i64 = conn.query_row(
        "SELECT COALESCE(SUM(total),0) FROM ventas
         WHERE estado='completada' AND date(fecha,'localtime') >= date('now','localtime','-6 days')",
        [],
        |r| r.get(0),
    )?;
    let ventas_mes: i64 = conn.query_row(
        "SELECT COALESCE(SUM(total),0) FROM ventas
         WHERE estado='completada' AND strftime('%Y-%m',fecha,'localtime') = strftime('%Y-%m','now','localtime')",
        [],
        |r| r.get(0),
    )?;
    let numero_ventas_mes: i64 = conn.query_row(
        "SELECT COUNT(*) FROM ventas
         WHERE estado='completada' AND strftime('%Y-%m',fecha,'localtime') = strftime('%Y-%m','now','localtime')",
        [],
        |r| r.get(0),
    )?;
    let ticket_promedio_mes = if numero_ventas_mes > 0 {
        ventas_mes / numero_ventas_mes
    } else {
        0
    };

    // Aproximación: usa el costo ACTUAL del producto, no el costo histórico al
    // momento de la venta (no se registra costo en `venta_detalle`). Es la
    // simplificación habitual cuando no se lleva costeo histórico por venta.
    let utilidad_bruta_mes: i64 = conn.query_row(
        "SELECT COALESCE(SUM(vd.subtotal - (vd.cantidad * p.precio_costo)), 0)
         FROM venta_detalle vd
         JOIN ventas v ON v.id = vd.venta_id
         JOIN productos p ON p.id = vd.producto_id
         WHERE v.estado = 'completada' AND strftime('%Y-%m', v.fecha,'localtime') = strftime('%Y-%m','now','localtime')",
        [],
        |r| r.get(0),
    )?;

    let productos_stock_bajo: i64 = conn.query_row(
        "SELECT COUNT(*) FROM productos WHERE estado='activo' AND stock_actual <= stock_minimo",
        [],
        |r| r.get(0),
    )?;

    let productos_sobre_stock: i64 = conn.query_row(
        "SELECT COUNT(*) FROM productos
         WHERE estado='activo' AND stock_maximo IS NOT NULL AND stock_actual > stock_maximo",
        [],
        |r| r.get(0),
    )?;

    let valor_inventario: i64 = conn.query_row(
        "SELECT COALESCE(SUM(stock_actual * precio_costo),0) FROM productos WHERE estado='activo'",
        [],
        |r| r.get(0),
    )?;

    Ok(DashboardIndicadores {
        ventas_hoy,
        ventas_semana,
        ventas_mes,
        numero_ventas_mes,
        ticket_promedio_mes,
        utilidad_bruta_mes,
        productos_stock_bajo,
        productos_sobre_stock,
        valor_inventario,
    })
}

fn dias_entre(desde: &str, hasta: &str) -> AppResult<i64> {
    let d = chrono::NaiveDate::parse_from_str(desde, "%Y-%m-%d")
        .map_err(|_| AppError::Validacion("Fecha 'desde' inválida.".into()))?;
    let h = chrono::NaiveDate::parse_from_str(hasta, "%Y-%m-%d")
        .map_err(|_| AppError::Validacion("Fecha 'hasta' inválida.".into()))?;
    Ok((h - d).num_days().max(0) + 1)
}

fn periodo_anterior(desde: &str, dias: i64) -> AppResult<(String, String)> {
    let d = chrono::NaiveDate::parse_from_str(desde, "%Y-%m-%d")
        .map_err(|_| AppError::Validacion("Fecha 'desde' inválida.".into()))?;
    let hasta_ant = d - chrono::Duration::days(1);
    let desde_ant = hasta_ant - chrono::Duration::days(dias - 1);
    Ok((
        desde_ant.format("%Y-%m-%d").to_string(),
        hasta_ant.format("%Y-%m-%d").to_string(),
    ))
}

pub fn graficas(pool: &DbPool, desde: &str, hasta: &str) -> AppResult<DashboardGraficas> {
    let conn = pool.get()?;

    let mut stmt = conn.prepare(
        "SELECT date(fecha,'localtime'), SUM(total) FROM ventas
         WHERE estado='completada' AND date(fecha,'localtime') BETWEEN ?1 AND ?2
         GROUP BY date(fecha,'localtime') ORDER BY 1",
    )?;
    let serie_ventas = stmt
        .query_map(params![desde, hasta], |r| {
            Ok(PuntoSerie {
                etiqueta: r.get(0)?,
                total: r.get(1)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    let mut stmt = conn.prepare(
        "SELECT p.nombre, SUM(vd.cantidad) AS cant FROM venta_detalle vd
         JOIN ventas v ON v.id = vd.venta_id JOIN productos p ON p.id = vd.producto_id
         WHERE v.estado='completada' AND date(v.fecha,'localtime') BETWEEN ?1 AND ?2
         GROUP BY vd.producto_id ORDER BY cant DESC LIMIT 10",
    )?;
    let productos_mas_vendidos = stmt
        .query_map(params![desde, hasta], |r| {
            Ok(ProductoCantidad {
                nombre: r.get(0)?,
                cantidad: r.get(1)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    let mut stmt = conn.prepare(
        "SELECT p.nombre, SUM(vd.cantidad) AS cant FROM venta_detalle vd
         JOIN ventas v ON v.id = vd.venta_id JOIN productos p ON p.id = vd.producto_id
         WHERE v.estado='completada' AND date(v.fecha,'localtime') BETWEEN ?1 AND ?2
         GROUP BY vd.producto_id ORDER BY cant ASC LIMIT 10",
    )?;
    let productos_menos_vendidos = stmt
        .query_map(params![desde, hasta], |r| {
            Ok(ProductoCantidad {
                nombre: r.get(0)?,
                cantidad: r.get(1)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    let mut stmt = conn.prepare(
        "SELECT COALESCE(c.nombre,'Sin categoría'), SUM(vd.subtotal) FROM venta_detalle vd
         JOIN ventas v ON v.id=vd.venta_id JOIN productos p ON p.id=vd.producto_id
         LEFT JOIN categorias c ON c.id = p.categoria_id
         WHERE v.estado='completada' AND date(v.fecha,'localtime') BETWEEN ?1 AND ?2
         GROUP BY p.categoria_id ORDER BY 2 DESC",
    )?;
    let ventas_por_categoria = stmt
        .query_map(params![desde, hasta], |r| {
            Ok(PuntoSerie {
                etiqueta: r.get(0)?,
                total: r.get(1)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    let mut stmt = conn.prepare(
        "SELECT metodo_pago, SUM(total) FROM ventas
         WHERE estado='completada' AND date(fecha,'localtime') BETWEEN ?1 AND ?2 GROUP BY metodo_pago",
    )?;
    let ventas_por_metodo_pago = stmt
        .query_map(params![desde, hasta], |r| {
            Ok(PuntoSerie {
                etiqueta: r.get(0)?,
                total: r.get(1)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    let dias = dias_entre(desde, hasta)?;
    let (desde_ant, hasta_ant) = periodo_anterior(desde, dias)?;
    let actual: i64 = conn.query_row(
        "SELECT COALESCE(SUM(total),0) FROM ventas WHERE estado='completada' AND date(fecha,'localtime') BETWEEN ?1 AND ?2",
        params![desde, hasta],
        |r| r.get(0),
    )?;
    let anterior: i64 = conn.query_row(
        "SELECT COALESCE(SUM(total),0) FROM ventas WHERE estado='completada' AND date(fecha,'localtime') BETWEEN ?1 AND ?2",
        params![desde_ant, hasta_ant],
        |r| r.get(0),
    )?;

    Ok(DashboardGraficas {
        serie_ventas,
        productos_mas_vendidos,
        productos_menos_vendidos,
        ventas_por_categoria,
        ventas_por_metodo_pago,
        comparativo: Comparativo { actual, anterior },
    })
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

    fn crear_producto(pool: &DbPool, sku: &str, costo: i64, precio: i64, stock: i64) -> i64 {
        let conn = pool.get().unwrap();
        conn.execute(
            "INSERT INTO productos (sku, nombre, unidad_medida, precio_costo, precio_venta, stock_actual, stock_minimo)
             VALUES (?1, 'Producto', 'unidad', ?2, ?3, ?4, 5)",
            params![sku, costo, precio, stock],
        )
        .unwrap();
        conn.last_insert_rowid()
    }

    #[test]
    fn indicadores_reflejan_ventas_y_stock_bajo() {
        let pool = pool_de_prueba();
        let producto_id = crear_producto(&pool, "SKU-D1", 1000, 2000, 2); // stock 2 <= minimo 5

        venta_service::crear(
            &pool,
            VentaNueva {
                cliente_id: None,
                metodo_pago: "tarjeta".into(),
                valor_recibido: None,
                descuento_global: 0,
                items: vec![VentaItemNuevo {
                    producto_id,
                    cantidad: 1,
                    precio_unitario: 2000,
                    descuento_pct: 0.0,
                    descuento_valor: 0,
                    impuesto_pct: 0.0,
                }],
            },
        )
        .unwrap();

        let ind = indicadores(&pool).unwrap();
        assert_eq!(ind.ventas_hoy, 2000);
        assert_eq!(ind.ventas_mes, 2000);
        assert_eq!(ind.numero_ventas_mes, 1);
        assert_eq!(ind.ticket_promedio_mes, 2000);
        assert_eq!(ind.utilidad_bruta_mes, 1000); // 2000 venta - 1000 costo
        assert_eq!(ind.productos_stock_bajo, 1);
        // La venta ya descontó 1 unidad del stock inicial (2), queda 1 unidad a 1000 c/u.
        assert_eq!(ind.valor_inventario, 1000);
    }

    #[test]
    fn indicadores_cuenta_productos_en_sobre_stock() {
        let pool = pool_de_prueba();
        // Sin stock_maximo: nunca debe contar como sobre-stock aunque tenga mucho.
        crear_producto(&pool, "SKU-D3", 1000, 2000, 500);

        let conn = pool.get().unwrap();
        conn.execute(
            "INSERT INTO productos (sku, nombre, unidad_medida, precio_costo, precio_venta,
                                     stock_actual, stock_minimo, stock_maximo)
             VALUES ('SKU-D4', 'Producto', 'unidad', 1000, 2000, 50, 5, 20)",
            [],
        )
        .unwrap();
        drop(conn);

        let ind = indicadores(&pool).unwrap();
        assert_eq!(ind.productos_sobre_stock, 1);
    }

    #[test]
    fn graficas_agrupan_por_dia_y_calculan_comparativo() {
        let pool = pool_de_prueba();
        let producto_id = crear_producto(&pool, "SKU-D2", 1000, 2000, 100);

        venta_service::crear(
            &pool,
            VentaNueva {
                cliente_id: None,
                metodo_pago: "tarjeta".into(),
                valor_recibido: None,
                descuento_global: 0,
                items: vec![VentaItemNuevo {
                    producto_id,
                    cantidad: 3,
                    precio_unitario: 2000,
                    descuento_pct: 0.0,
                    descuento_valor: 0,
                    impuesto_pct: 0.0,
                }],
            },
        )
        .unwrap();

        let hoy = chrono::Local::now().format("%Y-%m-%d").to_string();
        let graficas = graficas(&pool, &hoy, &hoy).unwrap();

        assert_eq!(graficas.serie_ventas.len(), 1);
        assert_eq!(graficas.serie_ventas[0].total, 6000);
        assert_eq!(graficas.productos_mas_vendidos[0].cantidad, 3);
        assert_eq!(graficas.comparativo.actual, 6000);
        assert_eq!(graficas.comparativo.anterior, 0);
    }
}

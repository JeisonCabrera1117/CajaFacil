use rusqlite::params;
use rusqlite::types::Value;

use crate::db::DbPool;
use crate::error::AppResult;
use crate::models::reporte::{
    FilaComprasProveedor, FilaInventarioValorizado, FilaReporteVenta, FilaRotacion, FilaUtilidad,
    FiltroReporteVentas,
};

pub fn ventas(pool: &DbPool, filtro: &FiltroReporteVentas) -> AppResult<Vec<FilaReporteVenta>> {
    let conn = pool.get()?;

    let mut condiciones = vec!["v.estado != 'anulada'".to_string()];
    let mut valores: Vec<Value> = Vec::new();

    if let Some(d) = filtro.desde.as_deref().filter(|s| !s.is_empty()) {
        condiciones.push("date(v.fecha,'localtime') >= ?".into());
        valores.push(Value::Text(d.to_string()));
    }
    if let Some(h) = filtro.hasta.as_deref().filter(|s| !s.is_empty()) {
        condiciones.push("date(v.fecha,'localtime') <= ?".into());
        valores.push(Value::Text(h.to_string()));
    }
    if let Some(p) = filtro.producto_id {
        condiciones.push("vd.producto_id = ?".into());
        valores.push(Value::Integer(p));
    }
    if let Some(c) = filtro.categoria_id {
        condiciones.push("p.categoria_id = ?".into());
        valores.push(Value::Integer(c));
    }
    if let Some(cl) = filtro.cliente_id {
        condiciones.push("v.cliente_id = ?".into());
        valores.push(Value::Integer(cl));
    }

    let sql = format!(
        "SELECT v.fecha, v.numero_comprobante, COALESCE(cl.nombre,'Cliente general'), p.nombre,
                cat.nombre, vd.cantidad, vd.precio_unitario, vd.subtotal, v.estado
         FROM venta_detalle vd
         JOIN ventas v ON v.id = vd.venta_id
         JOIN productos p ON p.id = vd.producto_id
         LEFT JOIN categorias cat ON cat.id = p.categoria_id
         LEFT JOIN clientes cl ON cl.id = v.cliente_id
         WHERE {}
         ORDER BY v.fecha DESC",
        condiciones.join(" AND ")
    );
    let mut stmt = conn.prepare(&sql)?;
    let filas = stmt.query_map(rusqlite::params_from_iter(valores.iter()), |r| {
        Ok(FilaReporteVenta {
            fecha: r.get(0)?,
            numero_comprobante: r.get(1)?,
            cliente: r.get(2)?,
            producto: r.get(3)?,
            categoria: r.get(4)?,
            cantidad: r.get(5)?,
            precio_unitario: r.get(6)?,
            subtotal: r.get(7)?,
            estado: r.get(8)?,
        })
    })?;
    let mut out = Vec::new();
    for f in filas {
        out.push(f?);
    }
    Ok(out)
}

/// Misma aproximación de costo actual (no histórico) que `dashboard_service::indicadores`.
pub fn utilidad(pool: &DbPool, desde: &str, hasta: &str) -> AppResult<Vec<FilaUtilidad>> {
    let conn = pool.get()?;
    let mut stmt = conn.prepare(
        "SELECT p.nombre, SUM(vd.cantidad), SUM(vd.subtotal), SUM(vd.cantidad * p.precio_costo)
         FROM venta_detalle vd
         JOIN ventas v ON v.id = vd.venta_id
         JOIN productos p ON p.id = vd.producto_id
         WHERE v.estado = 'completada' AND date(v.fecha,'localtime') BETWEEN ?1 AND ?2
         GROUP BY vd.producto_id ORDER BY 3 DESC",
    )?;
    let filas = stmt.query_map(params![desde, hasta], |r| {
        let ingresos: i64 = r.get(2)?;
        let costo: i64 = r.get(3)?;
        Ok(FilaUtilidad {
            producto: r.get(0)?,
            cantidad_vendida: r.get(1)?,
            ingresos,
            costo,
            utilidad: ingresos - costo,
        })
    })?;
    let mut out = Vec::new();
    for f in filas {
        out.push(f?);
    }
    Ok(out)
}

pub fn inventario_valorizado(pool: &DbPool) -> AppResult<Vec<FilaInventarioValorizado>> {
    let conn = pool.get()?;
    let mut stmt = conn.prepare(
        "SELECT p.sku, p.nombre, c.nombre, p.stock_actual, p.precio_costo, p.stock_actual * p.precio_costo
         FROM productos p LEFT JOIN categorias c ON c.id = p.categoria_id
         WHERE p.estado = 'activo' ORDER BY p.nombre",
    )?;
    let filas = stmt.query_map([], |r| {
        Ok(FilaInventarioValorizado {
            sku: r.get(0)?,
            nombre: r.get(1)?,
            categoria: r.get(2)?,
            stock_actual: r.get(3)?,
            costo_unitario: r.get(4)?,
            valor_total: r.get(5)?,
        })
    })?;
    let mut out = Vec::new();
    for f in filas {
        out.push(f?);
    }
    Ok(out)
}

/// "Sin movimiento" mira CUALQUIER movimiento de stock en el rango (entrada,
/// salida, ajuste, devolución), no solo ventas — un producto que solo tuvo
/// ajustes de inventario tampoco cuenta como "sin rotación real", pero sí
/// evita el falso positivo de marcarlo "sin movimiento" cuando en realidad
/// se le hizo un conteo o una entrada.
pub fn rotacion(pool: &DbPool, desde: &str, hasta: &str) -> AppResult<Vec<FilaRotacion>> {
    let conn = pool.get()?;
    let mut stmt = conn.prepare(
        "SELECT p.sku, p.nombre, p.stock_actual,
                COALESCE((SELECT SUM(vd.cantidad) FROM venta_detalle vd JOIN ventas v ON v.id = vd.venta_id
                          WHERE vd.producto_id = p.id AND v.estado = 'completada'
                                AND date(v.fecha,'localtime') BETWEEN ?1 AND ?2), 0) AS vendido,
                (SELECT COUNT(*) FROM movimientos_stock m
                 WHERE m.producto_id = p.id AND date(m.fecha,'localtime') BETWEEN ?1 AND ?2) AS movimientos
         FROM productos p WHERE p.estado = 'activo' ORDER BY vendido ASC, p.nombre",
    )?;
    let filas = stmt.query_map(params![desde, hasta], |r| {
        let movimientos: i64 = r.get(4)?;
        Ok(FilaRotacion {
            sku: r.get(0)?,
            nombre: r.get(1)?,
            stock_actual: r.get(2)?,
            cantidad_vendida: r.get(3)?,
            sin_movimiento: movimientos == 0,
        })
    })?;
    let mut out = Vec::new();
    for f in filas {
        out.push(f?);
    }
    Ok(out)
}

pub fn compras_por_proveedor(
    pool: &DbPool,
    desde: &str,
    hasta: &str,
) -> AppResult<Vec<FilaComprasProveedor>> {
    let conn = pool.get()?;
    let mut stmt = conn.prepare(
        "SELECT COALESCE(pr.razon_social, 'Sin proveedor'), COUNT(*), SUM(c.total)
         FROM compras c LEFT JOIN proveedores pr ON pr.id = c.proveedor_id
         WHERE c.estado = 'registrada' AND date(c.fecha,'localtime') BETWEEN ?1 AND ?2
         GROUP BY c.proveedor_id ORDER BY 3 DESC",
    )?;
    let filas = stmt.query_map(params![desde, hasta], |r| {
        Ok(FilaComprasProveedor {
            proveedor: r.get(0)?,
            numero_compras: r.get(1)?,
            total_comprado: r.get(2)?,
        })
    })?;
    let mut out = Vec::new();
    for f in filas {
        out.push(f?);
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::compra::{CompraItemNuevo, CompraNueva};
    use crate::models::venta::{VentaItemNuevo, VentaNueva};
    use crate::services::{compra_service, venta_service};
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
            "INSERT INTO productos (sku, nombre, unidad_medida, precio_costo, precio_venta, stock_actual)
             VALUES (?1, 'Producto', 'unidad', ?2, ?3, ?4)",
            params![sku, costo, precio, stock],
        )
        .unwrap();
        conn.last_insert_rowid()
    }

    fn hoy() -> String {
        chrono::Local::now().format("%Y-%m-%d").to_string()
    }

    #[test]
    fn reporte_ventas_filtra_por_producto() {
        let pool = pool_de_prueba();
        let p1 = crear_producto(&pool, "SKU-R1", 1000, 2000, 10);
        let p2 = crear_producto(&pool, "SKU-R2", 1000, 2000, 10);
        for pid in [p1, p2] {
            venta_service::crear(
                &pool,
                VentaNueva {
                    cliente_id: None,
                    metodo_pago: "tarjeta".into(),
                    valor_recibido: None,
                    descuento_global: 0,
                    items: vec![VentaItemNuevo {
                        producto_id: pid,
                        cantidad: 1,
                        precio_unitario: 2000,
                        descuento_pct: 0.0,
                        descuento_valor: 0,
                        impuesto_pct: 0.0,
                    }],
                },
            )
            .unwrap();
        }

        let filas = ventas(
            &pool,
            &FiltroReporteVentas {
                producto_id: Some(p1),
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(filas.len(), 1);
        assert_eq!(filas[0].producto, "Producto");
    }

    #[test]
    fn reporte_utilidad_calcula_diferencia_ingreso_costo() {
        let pool = pool_de_prueba();
        let producto_id = crear_producto(&pool, "SKU-R3", 1000, 2000, 10);
        venta_service::crear(
            &pool,
            VentaNueva {
                cliente_id: None,
                metodo_pago: "tarjeta".into(),
                valor_recibido: None,
                descuento_global: 0,
                items: vec![VentaItemNuevo {
                    producto_id,
                    cantidad: 2,
                    precio_unitario: 2000,
                    descuento_pct: 0.0,
                    descuento_valor: 0,
                    impuesto_pct: 0.0,
                }],
            },
        )
        .unwrap();

        let dia = hoy();
        let filas = utilidad(&pool, &dia, &dia).unwrap();
        assert_eq!(filas.len(), 1);
        assert_eq!(filas[0].ingresos, 4000);
        assert_eq!(filas[0].costo, 2000);
        assert_eq!(filas[0].utilidad, 2000);
    }

    #[test]
    fn inventario_valorizado_multiplica_stock_por_costo() {
        let pool = pool_de_prueba();
        crear_producto(&pool, "SKU-R4", 1500, 3000, 4);
        let filas = inventario_valorizado(&pool).unwrap();
        assert_eq!(filas.len(), 1);
        assert_eq!(filas[0].valor_total, 6000);
    }

    #[test]
    fn rotacion_marca_sin_movimiento_correctamente() {
        let pool = pool_de_prueba();
        let vendido = crear_producto(&pool, "SKU-R5", 1000, 2000, 10);
        let quieto = crear_producto(&pool, "SKU-R6", 1000, 2000, 10);
        venta_service::crear(
            &pool,
            VentaNueva {
                cliente_id: None,
                metodo_pago: "tarjeta".into(),
                valor_recibido: None,
                descuento_global: 0,
                items: vec![VentaItemNuevo {
                    producto_id: vendido,
                    cantidad: 1,
                    precio_unitario: 2000,
                    descuento_pct: 0.0,
                    descuento_valor: 0,
                    impuesto_pct: 0.0,
                }],
            },
        )
        .unwrap();

        let dia = hoy();
        let filas = rotacion(&pool, &dia, &dia).unwrap();
        let fila_vendido = filas.iter().find(|f| f.sku == "SKU-R5").unwrap();
        let fila_quieto = filas.iter().find(|f| f.sku == "SKU-R6").unwrap();
        assert!(!fila_vendido.sin_movimiento);
        assert!(fila_quieto.sin_movimiento);
        let _ = quieto;
    }

    #[test]
    fn compras_por_proveedor_agrupa_totales() {
        let pool = pool_de_prueba();
        let producto_id = crear_producto(&pool, "SKU-R7", 1000, 2000, 0);
        let proveedor_id: i64 = {
            let conn = pool.get().unwrap();
            conn.execute(
                "INSERT INTO proveedores (razon_social) VALUES ('Proveedor Uno')",
                [],
            )
            .unwrap();
            conn.last_insert_rowid()
        };

        compra_service::crear(
            &pool,
            CompraNueva {
                proveedor_id: Some(proveedor_id),
                numero: "C-1".into(),
                observaciones: None,
                items: vec![CompraItemNuevo {
                    producto_id,
                    cantidad: 5,
                    costo_unitario: 1000,
                }],
            },
        )
        .unwrap();

        let dia = hoy();
        let filas = compras_por_proveedor(&pool, &dia, &dia).unwrap();
        assert_eq!(filas.len(), 1);
        assert_eq!(filas[0].proveedor, "Proveedor Uno");
        assert_eq!(filas[0].numero_compras, 1);
        assert_eq!(filas[0].total_comprado, 5000);
    }
}

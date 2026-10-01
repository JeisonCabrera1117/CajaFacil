use rusqlite::types::Value;
use rusqlite::{params, OptionalExtension};

use crate::db::DbPool;
use crate::error::{AppError, AppResult};
use crate::models::producto::Producto;
use crate::models::venta::{Venta, VentaConDetalle, VentaDetalleItem, VentaFiltro, VentaNueva};
use crate::services::audit_service;
use crate::services::producto_service;
use crate::services::stock_service::{self, NuevoMovimiento};

const METODOS_VALIDOS: [&str; 4] = ["efectivo", "tarjeta", "transferencia", "mixto"];

pub fn buscar_producto(pool: &DbPool, termino: &str) -> AppResult<Vec<Producto>> {
    let conn = pool.get()?;
    let patron = format!("%{}%", termino.trim());
    let sql = format!(
        "SELECT {} FROM productos
         WHERE estado = 'activo' AND (nombre LIKE ?1 OR sku LIKE ?1 OR codigo_barras LIKE ?1)
         ORDER BY nombre LIMIT 20",
        producto_service::COLUMNAS
    );
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map([patron], producto_service::map)?;
    let mut out = Vec::new();
    for r in rows {
        out.push(r?);
    }
    Ok(out)
}

fn map_venta(row: &rusqlite::Row) -> rusqlite::Result<Venta> {
    Ok(Venta {
        id: row.get(0)?,
        numero_comprobante: row.get(1)?,
        fecha: row.get(2)?,
        cliente_id: row.get(3)?,
        cliente_nombre: row.get(4)?,
        subtotal: row.get(5)?,
        descuento_total: row.get(6)?,
        impuestos: row.get(7)?,
        total: row.get(8)?,
        metodo_pago: row.get(9)?,
        valor_recibido: row.get(10)?,
        cambio: row.get(11)?,
        estado: row.get(12)?,
        motivo_anulacion: row.get(13)?,
        caja_sesion_id: row.get(14)?,
    })
}

const SELECT_VENTA: &str = "SELECT v.id, v.numero_comprobante, v.fecha, v.cliente_id, c.nombre,
        v.subtotal, v.descuento_total, v.impuestos, v.total, v.metodo_pago, v.valor_recibido,
        v.cambio, v.estado, v.motivo_anulacion, v.caja_sesion_id
     FROM ventas v LEFT JOIN clientes c ON c.id = v.cliente_id";

pub fn obtener(pool: &DbPool, id: i64) -> AppResult<VentaConDetalle> {
    let conn = pool.get()?;
    let venta = conn
        .query_row(&format!("{SELECT_VENTA} WHERE v.id = ?1"), [id], map_venta)
        .optional()?
        .ok_or_else(|| AppError::NoEncontrado("La venta no existe.".into()))?;

    let mut stmt = conn.prepare(
        "SELECT vd.id, vd.producto_id, p.nombre, vd.cantidad, vd.precio_unitario, vd.descuento_pct,
                vd.descuento_valor, vd.impuesto_pct, vd.subtotal,
                COALESCE((SELECT SUM(dd.cantidad) FROM devolucion_detalle dd WHERE dd.venta_detalle_id = vd.id), 0)
         FROM venta_detalle vd JOIN productos p ON p.id = vd.producto_id
         WHERE vd.venta_id = ?1 ORDER BY vd.id",
    )?;
    let rows = stmt.query_map([id], |r| {
        Ok(VentaDetalleItem {
            id: r.get(0)?,
            producto_id: r.get(1)?,
            producto_nombre: r.get(2)?,
            cantidad: r.get(3)?,
            precio_unitario: r.get(4)?,
            descuento_pct: r.get(5)?,
            descuento_valor: r.get(6)?,
            impuesto_pct: r.get(7)?,
            subtotal: r.get(8)?,
            cantidad_devuelta: r.get(9)?,
        })
    })?;
    let mut items = Vec::new();
    for r in rows {
        items.push(r?);
    }

    Ok(VentaConDetalle { venta, items })
}

pub fn listar(pool: &DbPool, filtro: VentaFiltro) -> AppResult<Vec<Venta>> {
    let conn = pool.get()?;

    let mut condiciones = vec!["1=1".to_string()];
    let mut valores: Vec<Value> = Vec::new();

    // `fecha` se guarda en UTC; comparar contra el día calendario local que
    // el usuario eligió requiere 'localtime' (ver nota en dashboard_service.rs).
    if let Some(desde) = filtro.desde.filter(|s| !s.is_empty()) {
        condiciones.push("date(v.fecha,'localtime') >= ?".into());
        valores.push(Value::Text(desde));
    }
    if let Some(hasta) = filtro.hasta.filter(|s| !s.is_empty()) {
        condiciones.push("date(v.fecha,'localtime') <= ?".into());
        valores.push(Value::Text(hasta));
    }
    if let Some(cliente_id) = filtro.cliente_id {
        condiciones.push("v.cliente_id = ?".into());
        valores.push(Value::Integer(cliente_id));
    }
    if let Some(estado) = filtro.estado.filter(|s| !s.is_empty()) {
        condiciones.push("v.estado = ?".into());
        valores.push(Value::Text(estado));
    }
    if let Some(metodo_pago) = filtro.metodo_pago.filter(|s| !s.is_empty()) {
        condiciones.push("v.metodo_pago = ?".into());
        valores.push(Value::Text(metodo_pago));
    }

    let sql = format!(
        "{SELECT_VENTA} WHERE {} ORDER BY v.fecha DESC",
        condiciones.join(" AND ")
    );
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map(rusqlite::params_from_iter(valores.iter()), map_venta)?;
    let mut out = Vec::new();
    for r in rows {
        out.push(r?);
    }
    Ok(out)
}

fn traducir_error(e: rusqlite::Error) -> AppError {
    if let rusqlite::Error::SqliteFailure(_, Some(msg)) = &e {
        if msg.contains("UNIQUE") {
            return AppError::Interno(
                "Colisión generando el número de comprobante; intente de nuevo.".into(),
            );
        }
    }
    AppError::Db(e)
}

pub fn crear(pool: &DbPool, datos: VentaNueva) -> AppResult<VentaConDetalle> {
    if !METODOS_VALIDOS.contains(&datos.metodo_pago.as_str()) {
        return Err(AppError::Validacion("Método de pago inválido.".into()));
    }
    if datos.items.is_empty() {
        return Err(AppError::Validacion(
            "La venta debe tener al menos un producto.".into(),
        ));
    }
    if datos.descuento_global < 0 {
        return Err(AppError::Validacion(
            "El descuento global no puede ser negativo.".into(),
        ));
    }
    for item in &datos.items {
        if item.cantidad <= 0 {
            return Err(AppError::Validacion(
                "La cantidad de cada línea debe ser mayor que cero.".into(),
            ));
        }
        if item.precio_unitario < 0 || item.descuento_valor < 0 {
            return Err(AppError::Validacion(
                "Los precios y descuentos no pueden ser negativos.".into(),
            ));
        }
    }

    let mut conn = pool.get()?;
    let tx = conn.transaction()?;

    let (prefijo, siguiente, arqueo_activo): (String, i64, i64) = tx.query_row(
        "SELECT prefijo_comprobante, siguiente_numero, arqueo_activo FROM empresa_config WHERE id = 1",
        [],
        |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
    )?;

    let caja_sesion_id: Option<i64> = if arqueo_activo != 0 {
        let abierta: Option<i64> = tx
            .query_row(
                "SELECT id FROM caja_sesiones WHERE fecha_cierre IS NULL",
                [],
                |r| r.get(0),
            )
            .optional()?;
        Some(abierta.ok_or_else(|| {
            AppError::Validacion("Debe abrir la caja antes de registrar ventas.".into())
        })?)
    } else {
        None
    };

    let mut subtotal = 0i64;
    let mut descuento_lineas = 0i64;
    let mut impuestos = 0i64;
    let mut netos = Vec::with_capacity(datos.items.len());
    for item in &datos.items {
        let bruto = item.cantidad * item.precio_unitario;
        let neto = bruto - item.descuento_valor;
        if neto < 0 {
            return Err(AppError::Validacion(
                "El descuento de una línea no puede superar su subtotal.".into(),
            ));
        }
        let impuesto_linea = ((neto as f64) * item.impuesto_pct / 100.0).round() as i64;
        subtotal += bruto;
        descuento_lineas += item.descuento_valor;
        impuestos += impuesto_linea;
        netos.push((neto, impuesto_linea));
    }
    let descuento_total = descuento_lineas + datos.descuento_global;
    let total = subtotal - descuento_total + impuestos;
    if total < 0 {
        return Err(AppError::Validacion(
            "El descuento total no puede superar el valor de la venta.".into(),
        ));
    }

    let (valor_recibido, cambio) = if datos.metodo_pago == "efectivo" {
        let recibido = datos.valor_recibido.ok_or_else(|| {
            AppError::Validacion("Debe indicar el valor recibido en efectivo.".into())
        })?;
        if recibido < total {
            return Err(AppError::Validacion(
                "El valor recibido es menor que el total de la venta.".into(),
            ));
        }
        (Some(recibido), Some(recibido - total))
    } else {
        (datos.valor_recibido, None)
    };

    let numero_comprobante = format!("{prefijo}-{siguiente:06}");
    tx.execute(
        "UPDATE empresa_config SET siguiente_numero = siguiente_numero + 1 WHERE id = 1",
        [],
    )?;

    tx.execute(
        "INSERT INTO ventas (numero_comprobante, cliente_id, subtotal, descuento_total, impuestos,
            total, metodo_pago, valor_recibido, cambio, estado, caja_sesion_id)
         VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,'completada',?10)",
        params![
            numero_comprobante,
            datos.cliente_id,
            subtotal,
            descuento_total,
            impuestos,
            total,
            datos.metodo_pago,
            valor_recibido,
            cambio,
            caja_sesion_id,
        ],
    )
    .map_err(traducir_error)?;
    let venta_id = tx.last_insert_rowid();

    for (item, (neto, impuesto_linea)) in datos.items.iter().zip(netos.iter()) {
        let subtotal_linea = neto + impuesto_linea;
        tx.execute(
            "INSERT INTO venta_detalle (venta_id, producto_id, cantidad, precio_unitario,
                descuento_pct, descuento_valor, impuesto_pct, subtotal)
             VALUES (?1,?2,?3,?4,?5,?6,?7,?8)",
            params![
                venta_id,
                item.producto_id,
                item.cantidad,
                item.precio_unitario,
                item.descuento_pct,
                item.descuento_valor,
                item.impuesto_pct,
                subtotal_linea,
            ],
        )?;

        stock_service::aplicar_movimiento_tx(
            &tx,
            NuevoMovimiento {
                producto_id: item.producto_id,
                tipo: "salida".into(),
                cantidad: item.cantidad,
                costo_unitario: None,
                motivo: None,
                referencia_tipo: Some("venta".into()),
                referencia_id: Some(venta_id),
            },
        )?;
    }

    tx.commit()?;
    drop(conn);

    let creada = obtener(pool, venta_id)?;
    audit_service::registrar(pool, "venta", Some(venta_id), "crear", &creada.venta)?;
    Ok(creada)
}

pub fn anular(pool: &DbPool, id: i64, motivo: String) -> AppResult<VentaConDetalle> {
    if motivo.trim().is_empty() {
        return Err(AppError::Validacion(
            "El motivo de anulación es obligatorio.".into(),
        ));
    }
    let existente = obtener(pool, id)?;
    if existente.venta.estado == "anulada" {
        return Err(AppError::Validacion("La venta ya está anulada.".into()));
    }
    if existente.items.iter().any(|i| i.cantidad_devuelta > 0) {
        return Err(AppError::Validacion(
            "Esta venta ya tiene devoluciones registradas; no se puede anular.".into(),
        ));
    }

    let mut conn = pool.get()?;
    let tx = conn.transaction()?;

    for item in &existente.items {
        stock_service::aplicar_movimiento_tx(
            &tx,
            NuevoMovimiento {
                producto_id: item.producto_id,
                tipo: "entrada".into(),
                cantidad: item.cantidad,
                costo_unitario: None,
                motivo: Some(format!(
                    "Anulación de venta {}: {motivo}",
                    existente.venta.numero_comprobante
                )),
                referencia_tipo: Some("anulacion_venta".into()),
                referencia_id: Some(id),
            },
        )?;
    }

    tx.execute(
        "UPDATE ventas SET estado = 'anulada', motivo_anulacion = ?1 WHERE id = ?2",
        params![motivo, id],
    )?;
    tx.commit()?;
    drop(conn);

    let actualizada = obtener(pool, id)?;
    audit_service::registrar(pool, "venta", Some(id), "anular", &actualizada.venta)?;
    Ok(actualizada)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::venta::VentaItemNuevo;
    use crate::services::stock_service;
    use tempfile::tempdir;

    fn pool_de_prueba() -> DbPool {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.sqlite3");
        let pool = crate::db::init_pool(&path).unwrap();
        std::mem::forget(dir);
        pool
    }

    fn crear_producto_con_stock(pool: &DbPool, sku: &str, precio_venta: i64, stock: i64) -> i64 {
        let conn = pool.get().unwrap();
        conn.execute(
            "INSERT INTO productos (sku, nombre, unidad_medida, precio_costo, precio_venta, stock_actual)
             VALUES (?1, 'Producto de prueba', 'unidad', 100, ?2, ?3)",
            params![sku, precio_venta, stock],
        )
        .unwrap();
        conn.last_insert_rowid()
    }

    fn item_simple(producto_id: i64, cantidad: i64, precio_unitario: i64) -> VentaItemNuevo {
        VentaItemNuevo {
            producto_id,
            cantidad,
            precio_unitario,
            descuento_pct: 0.0,
            descuento_valor: 0,
            impuesto_pct: 0.0,
        }
    }

    #[test]
    fn crear_venta_efectivo_calcula_cambio_baja_stock_y_numera() {
        let pool = pool_de_prueba();
        let producto_id = crear_producto_con_stock(&pool, "SKU-V1", 5000, 10);

        let venta = crear(
            &pool,
            VentaNueva {
                cliente_id: None,
                metodo_pago: "efectivo".into(),
                valor_recibido: Some(20_000),
                descuento_global: 0,
                items: vec![item_simple(producto_id, 2, 5000)],
            },
        )
        .unwrap();

        assert_eq!(venta.venta.total, 10_000);
        assert_eq!(venta.venta.cambio, Some(10_000));
        assert!(venta.venta.numero_comprobante.starts_with("CF-"));

        let conn = pool.get().unwrap();
        let stock: i64 = conn
            .query_row(
                "SELECT stock_actual FROM productos WHERE id = ?1",
                [producto_id],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(stock, 8);
    }

    #[test]
    fn rechaza_efectivo_insuficiente() {
        let pool = pool_de_prueba();
        let producto_id = crear_producto_con_stock(&pool, "SKU-V2", 5000, 10);

        let resultado = crear(
            &pool,
            VentaNueva {
                cliente_id: None,
                metodo_pago: "efectivo".into(),
                valor_recibido: Some(1_000),
                descuento_global: 0,
                items: vec![item_simple(producto_id, 1, 5000)],
            },
        );
        assert!(resultado.is_err());
    }

    #[test]
    fn anular_venta_revierte_stock() {
        let pool = pool_de_prueba();
        let producto_id = crear_producto_con_stock(&pool, "SKU-V3", 5000, 10);

        let venta = crear(
            &pool,
            VentaNueva {
                cliente_id: None,
                metodo_pago: "tarjeta".into(),
                valor_recibido: None,
                descuento_global: 0,
                items: vec![item_simple(producto_id, 3, 5000)],
            },
        )
        .unwrap();

        anular(&pool, venta.venta.id, "Cliente se arrepintió".into()).unwrap();

        let stock: i64 = {
            let conn = pool.get().unwrap();
            conn.query_row(
                "SELECT stock_actual FROM productos WHERE id = ?1",
                [producto_id],
                |r| r.get(0),
            )
            .unwrap()
        };
        assert_eq!(stock, 10);

        let actualizada = obtener(&pool, venta.venta.id).unwrap();
        assert_eq!(actualizada.venta.estado, "anulada");
    }

    #[test]
    fn requiere_caja_abierta_cuando_arqueo_esta_activo() {
        let pool = pool_de_prueba();
        let producto_id = crear_producto_con_stock(&pool, "SKU-V4", 5000, 10);
        {
            let conn = pool.get().unwrap();
            conn.execute(
                "UPDATE empresa_config SET arqueo_activo = 1 WHERE id = 1",
                [],
            )
            .unwrap();
        }

        let sin_caja = crear(
            &pool,
            VentaNueva {
                cliente_id: None,
                metodo_pago: "tarjeta".into(),
                valor_recibido: None,
                descuento_global: 0,
                items: vec![item_simple(producto_id, 1, 5000)],
            },
        );
        assert!(sin_caja.is_err());

        crate::services::caja_service::abrir(&pool, 50_000).unwrap();

        let con_caja = crear(
            &pool,
            VentaNueva {
                cliente_id: None,
                metodo_pago: "tarjeta".into(),
                valor_recibido: None,
                descuento_global: 0,
                items: vec![item_simple(producto_id, 1, 5000)],
            },
        );
        assert!(con_caja.is_ok());
    }

    #[test]
    fn no_deja_stock_negativo_al_vender() {
        let pool = pool_de_prueba();
        let producto_id = crear_producto_con_stock(&pool, "SKU-V5", 5000, 1);

        let resultado = crear(
            &pool,
            VentaNueva {
                cliente_id: None,
                metodo_pago: "tarjeta".into(),
                valor_recibido: None,
                descuento_global: 0,
                items: vec![item_simple(producto_id, 5, 5000)],
            },
        );
        assert!(resultado.is_err());

        // El stock no debe haber cambiado: la transacción se revierte completa.
        let stock = stock_service::kardex(&pool, producto_id).unwrap();
        assert!(stock.is_empty());
    }
}

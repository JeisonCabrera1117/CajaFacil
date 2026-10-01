use std::collections::HashMap;

use rusqlite::{params, Connection, OptionalExtension};

use crate::importacion::ayudas::{
    buscar_producto_por_sku, obtener_o_crear_cliente_por_documento, texto, traducir_error_sql,
};
use crate::importacion::{numeros, ResultadoFila};
use crate::models::importacion::CampoImport;
use crate::services::stock_service::{aplicar_movimiento_tx, NuevoMovimiento};

const METODOS_VALIDOS: [&str; 4] = ["efectivo", "tarjeta", "transferencia", "mixto"];

/// Cada fila del archivo representa una venta con un único producto (no se
/// pueden importar ventas con varios ítems desde CSV/XLSX) — simplificación
/// necesaria para que una fila = un registro, como en el resto de entidades.
pub fn campos() -> Vec<CampoImport> {
    vec![
        CampoImport::obligatorio("fecha", "Fecha (AAAA-MM-DD)"),
        CampoImport::opcional("numeroComprobante", "Número de comprobante"),
        CampoImport::opcional("clienteDocumento", "Documento del cliente"),
        CampoImport::obligatorio("sku", "SKU del producto"),
        CampoImport::obligatorio("cantidad", "Cantidad"),
        CampoImport::obligatorio("precioUnitario", "Precio unitario"),
        CampoImport::opcional("descuentoValor", "Descuento"),
        CampoImport::opcional("impuestoPct", "Impuesto (%)"),
        CampoImport::obligatorio("metodoPago", "Método de pago"),
        CampoImport::opcional("valorRecibido", "Valor recibido (si es efectivo)"),
    ]
}

fn normalizar_fecha(texto_fecha: &str) -> Option<String> {
    let t = texto_fecha.trim();
    if t.len() == 10 && t.as_bytes().get(4) == Some(&b'-') {
        return Some(format!("{t} 00:00:00"));
    }
    if t.len() >= 19 {
        return Some(t[..19].replace('T', " "));
    }
    None
}

fn generar_numero_comprobante(tx: &Connection) -> Result<String, String> {
    let (prefijo, siguiente): (String, i64) = tx
        .query_row(
            "SELECT prefijo_comprobante, siguiente_numero FROM empresa_config WHERE id = 1",
            [],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .map_err(|e| traducir_error_sql(&e))?;
    tx.execute(
        "UPDATE empresa_config SET siguiente_numero = siguiente_numero + 1 WHERE id = 1",
        [],
    )
    .map_err(|e| traducir_error_sql(&e))?;
    Ok(format!("{prefijo}-{siguiente:06}"))
}

pub fn procesar_fila(tx: &Connection, fila: &HashMap<String, String>) -> ResultadoFila {
    let Some(fecha_cruda) = texto(fila, "fecha") else {
        return ResultadoFila::Error("La fecha es obligatoria.".into());
    };
    let Some(fecha) = normalizar_fecha(&fecha_cruda) else {
        return ResultadoFila::Error(format!(
            "Fecha inválida: \"{fecha_cruda}\". Use AAAA-MM-DD."
        ));
    };
    let Some(sku) = texto(fila, "sku") else {
        return ResultadoFila::Error("El SKU es obligatorio.".into());
    };
    let Some(cantidad) = fila
        .get("cantidad")
        .and_then(|s| numeros::parsear_entero(s))
    else {
        return ResultadoFila::Error("La cantidad es obligatoria y debe ser numérica.".into());
    };
    if cantidad <= 0 {
        return ResultadoFila::Error("La cantidad debe ser mayor que cero.".into());
    }
    let Some(precio_unitario) = fila
        .get("precioUnitario")
        .and_then(|s| numeros::parsear_centavos(s))
    else {
        return ResultadoFila::Error(
            "El precio unitario es obligatorio y debe ser numérico.".into(),
        );
    };
    let descuento_valor = fila
        .get("descuentoValor")
        .and_then(|s| numeros::parsear_centavos(s))
        .unwrap_or(0);
    let impuesto_pct = fila
        .get("impuestoPct")
        .and_then(|s| numeros::parsear_numero(s))
        .unwrap_or(0.0);
    let Some(metodo_pago) = texto(fila, "metodoPago").map(|s| s.to_lowercase()) else {
        return ResultadoFila::Error("El método de pago es obligatorio.".into());
    };
    if !METODOS_VALIDOS.contains(&metodo_pago.as_str()) {
        return ResultadoFila::Error(format!("Método de pago inválido: \"{metodo_pago}\"."));
    }

    let producto_id = match buscar_producto_por_sku(tx, &sku) {
        Ok(Some(id)) => id,
        Ok(None) => {
            return ResultadoFila::Error(format!("No existe ningún producto con el SKU \"{sku}\"."))
        }
        Err(e) => return ResultadoFila::Error(e),
    };

    let cliente_id: Option<i64> = match texto(fila, "clienteDocumento") {
        Some(doc) => match obtener_o_crear_cliente_por_documento(tx, &doc, &doc) {
            Ok(id) => Some(id),
            Err(e) => return ResultadoFila::Error(e),
        },
        None => None,
    };

    let bruto = cantidad * precio_unitario;
    let neto = bruto - descuento_valor;
    if neto < 0 {
        return ResultadoFila::Error("El descuento no puede superar el subtotal.".into());
    }
    let impuesto = ((neto as f64) * impuesto_pct / 100.0).round() as i64;
    let subtotal_linea = neto + impuesto;
    let total = subtotal_linea;

    let valor_recibido_campo = fila
        .get("valorRecibido")
        .and_then(|s| numeros::parsear_centavos(s));
    let (valor_recibido, cambio) = if metodo_pago == "efectivo" {
        let Some(recibido) = valor_recibido_campo else {
            return ResultadoFila::Error(
                "El valor recibido es obligatorio cuando el método de pago es efectivo.".into(),
            );
        };
        if recibido < total {
            return ResultadoFila::Error("El valor recibido es menor que el total.".into());
        }
        (Some(recibido), Some(recibido - total))
    } else {
        (valor_recibido_campo, None)
    };

    let numero_comprobante = match texto(fila, "numeroComprobante") {
        Some(n) => n,
        None => match generar_numero_comprobante(tx) {
            Ok(n) => n,
            Err(e) => return ResultadoFila::Error(e),
        },
    };

    let ya_existe: Option<i64> = tx
        .query_row(
            "SELECT id FROM ventas WHERE numero_comprobante = ?1",
            [&numero_comprobante],
            |r| r.get(0),
        )
        .optional()
        .unwrap_or(None);
    if ya_existe.is_some() {
        return ResultadoFila::Omitido(format!(
            "Ya existe una venta con el comprobante \"{numero_comprobante}\"."
        ));
    }

    let insercion = tx.execute(
        "INSERT INTO ventas (numero_comprobante, fecha, cliente_id, subtotal, descuento_total, impuestos,
            total, metodo_pago, valor_recibido, cambio, estado)
         VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,'completada')",
        params![
            numero_comprobante,
            fecha,
            cliente_id,
            bruto,
            descuento_valor,
            impuesto,
            total,
            metodo_pago,
            valor_recibido,
            cambio
        ],
    );
    let venta_id = match insercion {
        Ok(_) => tx.last_insert_rowid(),
        Err(e) => return ResultadoFila::Error(traducir_error_sql(&e)),
    };

    if let Err(e) = tx.execute(
        "INSERT INTO venta_detalle (venta_id, producto_id, cantidad, precio_unitario, descuento_pct,
            descuento_valor, impuesto_pct, subtotal)
         VALUES (?1,?2,?3,?4,0,?5,?6,?7)",
        params![venta_id, producto_id, cantidad, precio_unitario, descuento_valor, impuesto_pct, subtotal_linea],
    ) {
        return ResultadoFila::Error(traducir_error_sql(&e));
    }

    match aplicar_movimiento_tx(
        tx,
        NuevoMovimiento {
            producto_id,
            tipo: "salida".into(),
            cantidad,
            costo_unitario: None,
            motivo: None,
            referencia_tipo: Some("carga_masiva_venta".into()),
            referencia_id: Some(venta_id),
        },
    ) {
        Ok(_) => ResultadoFila::Creado,
        Err(e) => ResultadoFila::Error(e.to_string()),
    }
}

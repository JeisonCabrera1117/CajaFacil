use std::collections::HashMap;

use rusqlite::Connection;

use crate::importacion::ayudas::{buscar_producto_por_sku, texto};
use crate::importacion::{numeros, ResultadoFila};
use crate::models::importacion::CampoImport;
use crate::services::stock_service::{aplicar_movimiento_tx, NuevoMovimiento};

pub fn campos() -> Vec<CampoImport> {
    vec![
        CampoImport::obligatorio("sku", "SKU"),
        CampoImport::obligatorio("cantidad", "Cantidad"),
        CampoImport::opcional("costoUnitario", "Costo unitario"),
    ]
}

/// El stock inicial siempre se aplica como un movimiento de entrada nuevo
/// (no hay "modo" de upsert: reimportar el mismo archivo suma stock otra vez,
/// tal como hacer la misma entrada manual dos veces — es intencional, porque
/// en esta app el stock solo cambia por movimientos acumulados, nunca se
/// sobrescribe directamente).
pub fn procesar_fila(tx: &Connection, fila: &HashMap<String, String>) -> ResultadoFila {
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
    let costo_unitario = fila
        .get("costoUnitario")
        .and_then(|s| numeros::parsear_centavos(s));

    let producto_id = match buscar_producto_por_sku(tx, &sku) {
        Ok(Some(id)) => id,
        Ok(None) => {
            return ResultadoFila::Error(format!("No existe ningún producto con el SKU \"{sku}\"."))
        }
        Err(e) => return ResultadoFila::Error(e),
    };

    let resultado = aplicar_movimiento_tx(
        tx,
        NuevoMovimiento {
            producto_id,
            tipo: "entrada".into(),
            cantidad,
            costo_unitario,
            motivo: None,
            referencia_tipo: Some("carga_masiva".into()),
            referencia_id: None,
        },
    );

    match resultado {
        Ok(_) => ResultadoFila::Creado,
        Err(e) => ResultadoFila::Error(e.to_string()),
    }
}

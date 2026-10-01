use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MovimientoStock {
    pub id: i64,
    pub producto_id: i64,
    pub tipo: String,
    pub cantidad: i64,
    pub costo_unitario: Option<i64>,
    pub motivo: Option<String>,
    pub referencia_tipo: Option<String>,
    pub referencia_id: Option<i64>,
    pub fecha: String,
    pub saldo_resultante: i64,
}

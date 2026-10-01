use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Comprobante {
    pub id: i64,
    pub venta_id: i64,
    pub tipo: String,
    pub ruta_archivo: String,
    pub generado_en: String,
}

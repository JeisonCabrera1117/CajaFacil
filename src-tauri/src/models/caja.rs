use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CajaSesion {
    pub id: i64,
    pub fecha_apertura: String,
    pub fecha_cierre: Option<String>,
    pub monto_apertura: i64,
    pub monto_cierre_sistema: Option<i64>,
    pub monto_cierre_contado: Option<i64>,
    pub diferencia: Option<i64>,
    pub observaciones: Option<String>,
}

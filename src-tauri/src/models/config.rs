use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct EmpresaConfig {
    pub nombre: String,
    pub nit: String,
    pub direccion: String,
    pub telefono: String,
    pub logo_path: Option<String>,
    pub moneda: String,
    pub formato_fecha: String,
    pub prefijo_comprobante: String,
    pub siguiente_numero: i64,
    pub leyenda_pie: String,
    pub tema: String,
    pub requiere_pin: bool,
    pub permite_stock_negativo: bool,
    pub arqueo_activo: bool,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct EmpresaConfigActualizar {
    pub nombre: String,
    pub nit: String,
    pub direccion: String,
    pub telefono: String,
    pub logo_path: Option<String>,
    pub moneda: String,
    pub formato_fecha: String,
    pub prefijo_comprobante: String,
    pub leyenda_pie: String,
    pub tema: String,
    pub permite_stock_negativo: bool,
    pub arqueo_activo: bool,
}

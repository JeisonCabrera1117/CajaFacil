use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct InventarioFisico {
    pub id: i64,
    pub fecha: String,
    pub estado: String,
    pub observaciones: Option<String>,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct InventarioFisicoDetalle {
    pub id: i64,
    pub producto_id: i64,
    pub producto_nombre: String,
    pub sku: String,
    pub stock_sistema: i64,
    pub stock_contado: i64,
    pub diferencia: i64,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct InventarioFisicoConDetalle {
    pub cabecera: InventarioFisico,
    pub detalle: Vec<InventarioFisicoDetalle>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RegistrarConteo {
    pub inventario_fisico_id: i64,
    pub producto_id: i64,
    pub stock_contado: i64,
}

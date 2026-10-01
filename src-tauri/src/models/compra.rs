use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CompraItemNuevo {
    pub producto_id: i64,
    pub cantidad: i64,
    pub costo_unitario: i64,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CompraNueva {
    pub proveedor_id: Option<i64>,
    pub numero: String,
    pub observaciones: Option<String>,
    pub items: Vec<CompraItemNuevo>,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CompraDetalleItem {
    pub id: i64,
    pub producto_id: i64,
    pub producto_nombre: String,
    pub cantidad: i64,
    pub costo_unitario: i64,
    pub subtotal: i64,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Compra {
    pub id: i64,
    pub proveedor_id: Option<i64>,
    pub proveedor_nombre: Option<String>,
    pub numero: String,
    pub fecha: String,
    pub subtotal: i64,
    pub impuestos: i64,
    pub total: i64,
    pub estado: String,
    pub observaciones: Option<String>,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CompraConDetalle {
    #[serde(flatten)]
    pub compra: Compra,
    pub items: Vec<CompraDetalleItem>,
}

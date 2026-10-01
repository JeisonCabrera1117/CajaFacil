use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DevolucionItemNuevo {
    pub venta_detalle_id: i64,
    pub cantidad: i64,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DevolucionNueva {
    pub venta_id: i64,
    pub motivo: String,
    pub items: Vec<DevolucionItemNuevo>,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DevolucionDetalleItem {
    pub id: i64,
    pub venta_detalle_id: i64,
    pub producto_nombre: String,
    pub cantidad: i64,
    pub valor: i64,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Devolucion {
    pub id: i64,
    pub venta_id: i64,
    pub fecha: String,
    pub motivo: String,
    pub total_devuelto: i64,
    pub items: Vec<DevolucionDetalleItem>,
}

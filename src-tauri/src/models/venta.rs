use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VentaItemNuevo {
    pub producto_id: i64,
    pub cantidad: i64,
    pub precio_unitario: i64,
    pub descuento_pct: f64,
    pub descuento_valor: i64,
    pub impuesto_pct: f64,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VentaNueva {
    pub cliente_id: Option<i64>,
    pub metodo_pago: String,
    pub valor_recibido: Option<i64>,
    pub descuento_global: i64,
    pub items: Vec<VentaItemNuevo>,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VentaDetalleItem {
    pub id: i64,
    pub producto_id: i64,
    pub producto_nombre: String,
    pub cantidad: i64,
    pub precio_unitario: i64,
    pub descuento_pct: f64,
    pub descuento_valor: i64,
    pub impuesto_pct: f64,
    pub subtotal: i64,
    pub cantidad_devuelta: i64,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Venta {
    pub id: i64,
    pub numero_comprobante: String,
    pub fecha: String,
    pub cliente_id: Option<i64>,
    pub cliente_nombre: Option<String>,
    pub subtotal: i64,
    pub descuento_total: i64,
    pub impuestos: i64,
    pub total: i64,
    pub metodo_pago: String,
    pub valor_recibido: Option<i64>,
    pub cambio: Option<i64>,
    pub estado: String,
    pub motivo_anulacion: Option<String>,
    pub caja_sesion_id: Option<i64>,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VentaConDetalle {
    pub venta: Venta,
    pub items: Vec<VentaDetalleItem>,
}

// `default` a nivel de contenedor: el frontend manda este filtro con solo las claves
// que el usuario efectivamente eligió (ver la lección de `ProductoFiltro` en fase 2).
#[derive(Debug, Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct VentaFiltro {
    pub desde: Option<String>,
    pub hasta: Option<String>,
    pub cliente_id: Option<i64>,
    pub estado: Option<String>,
    pub metodo_pago: Option<String>,
}

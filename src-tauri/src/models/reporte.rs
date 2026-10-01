use serde::{Deserialize, Serialize};

// `default` a nivel de contenedor: mismo motivo que `ProductoFiltro`/`VentaFiltro`
// (el frontend manda solo las claves que el usuario efectivamente eligió).
#[derive(Debug, Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct FiltroReporteVentas {
    pub desde: Option<String>,
    pub hasta: Option<String>,
    pub producto_id: Option<i64>,
    pub categoria_id: Option<i64>,
    pub cliente_id: Option<i64>,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FilaReporteVenta {
    pub fecha: String,
    pub numero_comprobante: String,
    pub cliente: String,
    pub producto: String,
    pub categoria: Option<String>,
    pub cantidad: i64,
    pub precio_unitario: i64,
    pub subtotal: i64,
    pub estado: String,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FilaUtilidad {
    pub producto: String,
    pub cantidad_vendida: i64,
    pub ingresos: i64,
    pub costo: i64,
    pub utilidad: i64,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FilaInventarioValorizado {
    pub sku: String,
    pub nombre: String,
    pub categoria: Option<String>,
    pub stock_actual: i64,
    pub costo_unitario: i64,
    pub valor_total: i64,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FilaRotacion {
    pub sku: String,
    pub nombre: String,
    pub cantidad_vendida: i64,
    pub stock_actual: i64,
    pub sin_movimiento: bool,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FilaComprasProveedor {
    pub proveedor: String,
    pub numero_compras: i64,
    pub total_comprado: i64,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SolicitudExportacion {
    pub titulo: String,
    pub columnas: Vec<String>,
    pub filas: Vec<Vec<String>>,
    pub formato: String,
}

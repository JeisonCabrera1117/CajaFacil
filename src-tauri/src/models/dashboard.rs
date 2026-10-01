use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DashboardIndicadores {
    pub ventas_hoy: i64,
    pub ventas_semana: i64,
    pub ventas_mes: i64,
    pub numero_ventas_mes: i64,
    pub ticket_promedio_mes: i64,
    pub utilidad_bruta_mes: i64,
    pub productos_stock_bajo: i64,
    pub productos_sobre_stock: i64,
    pub valor_inventario: i64,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PuntoSerie {
    pub etiqueta: String,
    pub total: i64,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ProductoCantidad {
    pub nombre: String,
    pub cantidad: i64,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Comparativo {
    pub actual: i64,
    pub anterior: i64,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DashboardGraficas {
    pub serie_ventas: Vec<PuntoSerie>,
    pub productos_mas_vendidos: Vec<ProductoCantidad>,
    pub productos_menos_vendidos: Vec<ProductoCantidad>,
    pub ventas_por_categoria: Vec<PuntoSerie>,
    pub ventas_por_metodo_pago: Vec<PuntoSerie>,
    pub comparativo: Comparativo,
}

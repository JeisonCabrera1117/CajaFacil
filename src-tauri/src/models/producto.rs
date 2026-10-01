use serde::{Deserialize, Serialize};

/// `stock_actual` no se expone en `ProductoNuevo`: la única forma de cambiar
/// stock es a través de movimientos (ver `stock_service`), así el kardex
/// siempre cuadra con el saldo del producto.
#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Producto {
    pub id: i64,
    pub sku: String,
    pub codigo_barras: Option<String>,
    pub nombre: String,
    pub descripcion: Option<String>,
    pub categoria_id: Option<i64>,
    pub unidad_medida: String,
    pub precio_costo: i64,
    pub precio_venta: i64,
    pub impuesto_pct: f64,
    pub stock_actual: i64,
    pub stock_minimo: i64,
    pub stock_maximo: Option<i64>,
    pub ubicacion: Option<String>,
    pub proveedor_principal_id: Option<i64>,
    pub estado: String,
    pub imagen_path: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ProductoNuevo {
    pub sku: String,
    pub codigo_barras: Option<String>,
    pub nombre: String,
    pub descripcion: Option<String>,
    pub categoria_id: Option<i64>,
    pub unidad_medida: String,
    pub precio_costo: i64,
    pub precio_venta: i64,
    pub impuesto_pct: f64,
    pub stock_minimo: i64,
    pub stock_maximo: Option<i64>,
    pub ubicacion: Option<String>,
    pub proveedor_principal_id: Option<i64>,
    pub estado: String,
    pub imagen_path: Option<String>,
}

// `default` a nivel de contenedor: el frontend a veces manda el objeto con solo
// algunas claves (p. ej. al pedir el catálogo completo solo pasa paginación), y sin
// esto serde exige que TODAS las claves estén presentes aunque sean opcionales.
#[derive(Debug, Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct ProductoFiltro {
    pub busqueda: Option<String>,
    pub categoria_id: Option<i64>,
    pub solo_stock_bajo: Option<bool>,
    pub solo_sobre_stock: Option<bool>,
    pub pagina: Option<i64>,
    pub por_pagina: Option<i64>,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ProductoPagina {
    pub items: Vec<Producto>,
    pub total: i64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn filtro_acepta_json_con_claves_faltantes() {
        let vacio: ProductoFiltro = serde_json::from_str("{}").unwrap();
        assert_eq!(vacio.pagina, None);

        let parcial: ProductoFiltro =
            serde_json::from_str(r#"{"porPagina":500,"pagina":1}"#).unwrap();
        assert_eq!(parcial.por_pagina, Some(500));
        assert_eq!(parcial.pagina, Some(1));
        assert_eq!(parcial.busqueda, None);
    }
}

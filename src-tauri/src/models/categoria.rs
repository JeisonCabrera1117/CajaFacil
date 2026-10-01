use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Categoria {
    pub id: i64,
    pub nombre: String,
    pub categoria_padre_id: Option<i64>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CategoriaNueva {
    pub nombre: String,
    pub categoria_padre_id: Option<i64>,
}

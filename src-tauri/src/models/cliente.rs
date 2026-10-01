use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Cliente {
    pub id: i64,
    pub documento: Option<String>,
    pub nombre: String,
    pub telefono: Option<String>,
    pub correo: Option<String>,
    pub direccion: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ClienteNuevo {
    pub documento: Option<String>,
    pub nombre: String,
    pub telefono: Option<String>,
    pub correo: Option<String>,
    pub direccion: Option<String>,
}

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Proveedor {
    pub id: i64,
    pub nit: Option<String>,
    pub razon_social: String,
    pub contacto: Option<String>,
    pub telefono: Option<String>,
    pub correo: Option<String>,
    pub direccion: Option<String>,
    pub condiciones_pago: Option<String>,
    pub activo: bool,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ProveedorNuevo {
    pub nit: Option<String>,
    pub razon_social: String,
    pub contacto: Option<String>,
    pub telefono: Option<String>,
    pub correo: Option<String>,
    pub direccion: Option<String>,
    pub condiciones_pago: Option<String>,
    pub activo: bool,
}

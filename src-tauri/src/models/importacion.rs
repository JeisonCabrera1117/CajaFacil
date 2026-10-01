use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CampoImport {
    pub id: String,
    pub etiqueta: String,
    pub obligatorio: bool,
}

impl CampoImport {
    pub fn obligatorio(id: &str, etiqueta: &str) -> Self {
        Self {
            id: id.into(),
            etiqueta: etiqueta.into(),
            obligatorio: true,
        }
    }
    pub fn opcional(id: &str, etiqueta: &str) -> Self {
        Self {
            id: id.into(),
            etiqueta: etiqueta.into(),
            obligatorio: false,
        }
    }
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DeteccionArchivo {
    pub tipo: String,
    pub encoding: Option<String>,
    pub separador: Option<String>,
    pub hojas: Vec<String>,
    pub hoja_seleccionada: Option<String>,
    pub columnas: Vec<String>,
    pub filas_muestra: Vec<Vec<String>>,
    pub total_filas: i64,
}

#[derive(Debug, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ModoImport {
    Crear,
    Actualizar,
    CrearYActualizar,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct EntidadImportable {
    pub id: String,
    pub etiqueta: String,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FilaRechazada {
    pub fila: i64,
    pub motivo: String,
}

#[derive(Debug, Serialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct ResumenImportacion {
    pub total_filas: i64,
    pub creados: i64,
    pub actualizados: i64,
    pub omitidos: i64,
    pub con_error: i64,
    pub filas_rechazadas: Vec<FilaRechazada>,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Importacion {
    pub id: i64,
    pub entidad: String,
    pub archivo_nombre: String,
    pub fecha: String,
    pub modo: String,
    pub total_filas: i64,
    pub creados: i64,
    pub actualizados: i64,
    pub omitidos: i64,
    pub con_error: i64,
    pub archivo_rechazados_path: Option<String>,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PlantillaGenerada {
    pub nombre_archivo: String,
    pub contenido_base64: String,
    pub mime: String,
}

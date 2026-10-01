use std::collections::HashMap;

use tauri::{AppHandle, Emitter, State};

use crate::error::AppResult;
use crate::models::importacion::{
    CampoImport, DeteccionArchivo, EntidadImportable, Importacion, ModoImport, PlantillaGenerada,
    ResumenImportacion,
};
use crate::services::importacion_service;
use crate::state::AppState;

#[tauri::command]
pub fn import_entidades() -> Vec<EntidadImportable> {
    importacion_service::entidades_importables()
}

#[tauri::command]
pub fn import_campos(entidad: String) -> AppResult<Vec<CampoImport>> {
    importacion_service::campos_de(&entidad)
}

#[tauri::command]
pub fn import_plantilla_descargar(
    entidad: String,
    formato: String,
) -> AppResult<PlantillaGenerada> {
    importacion_service::plantilla(&entidad, &formato)
}

#[tauri::command]
pub fn import_detectar_archivo(
    ruta: String,
    hoja: Option<String>,
    separador: Option<String>,
) -> AppResult<DeteccionArchivo> {
    let separador_char = separador.and_then(|s| s.chars().next());
    importacion_service::detectar(&ruta, hoja.as_deref(), separador_char)
}

#[tauri::command]
pub fn import_preview(
    state: State<AppState>,
    ruta: String,
    hoja: Option<String>,
    separador: Option<String>,
    entidad: String,
    mapeo: HashMap<String, String>,
    modo: ModoImport,
) -> AppResult<ResumenImportacion> {
    let separador_char = separador.and_then(|s| s.chars().next());
    importacion_service::previsualizar(
        &state.pool,
        &ruta,
        hoja.as_deref(),
        separador_char,
        &entidad,
        &mapeo,
        modo,
    )
}

#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub fn import_ejecutar(
    app: AppHandle,
    state: State<AppState>,
    ruta: String,
    hoja: Option<String>,
    separador: Option<String>,
    entidad: String,
    mapeo: HashMap<String, String>,
    modo: ModoImport,
    nombre_archivo_original: String,
) -> AppResult<Importacion> {
    let separador_char = separador.and_then(|s| s.chars().next());
    importacion_service::ejecutar(
        &state.pool,
        &state.importaciones_dir,
        &ruta,
        hoja.as_deref(),
        separador_char,
        &entidad,
        &mapeo,
        modo,
        &nombre_archivo_original,
        |procesadas, total| {
            let _ = app.emit(
                "import-progreso",
                serde_json::json!({ "procesadas": procesadas, "total": total }),
            );
        },
    )
}

#[tauri::command]
pub fn import_historial(state: State<AppState>) -> AppResult<Vec<Importacion>> {
    importacion_service::historial(&state.pool)
}

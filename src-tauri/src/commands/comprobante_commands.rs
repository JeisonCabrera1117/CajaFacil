use tauri::State;

use crate::error::AppResult;
use crate::models::comprobante::Comprobante;
use crate::services::comprobante_service;
use crate::state::AppState;

#[tauri::command]
pub fn comprobante_generar(
    state: State<AppState>,
    venta_id: i64,
    formato: String,
) -> AppResult<Comprobante> {
    comprobante_service::generar(&state.pool, &state.comprobantes_dir, venta_id, &formato)
}

#[tauri::command]
pub fn comprobante_regenerar(
    state: State<AppState>,
    venta_id: i64,
    formato: String,
) -> AppResult<Comprobante> {
    comprobante_service::generar(&state.pool, &state.comprobantes_dir, venta_id, &formato)
}

#[tauri::command]
pub fn comprobante_abrir(ruta_archivo: String) -> AppResult<()> {
    comprobante_service::abrir(&ruta_archivo)
}

#[tauri::command]
pub fn comprobante_copiar_imagen(ruta_archivo: String) -> AppResult<()> {
    comprobante_service::copiar_imagen(&ruta_archivo)
}

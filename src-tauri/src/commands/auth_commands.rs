use tauri::State;

use crate::error::AppResult;
use crate::services::auth_service;
use crate::state::AppState;

#[tauri::command]
pub fn auth_check_required(state: State<AppState>) -> AppResult<bool> {
    auth_service::pin_requerido(&state.pool)
}

#[tauri::command]
pub fn auth_set_pin(state: State<AppState>, pin: String) -> AppResult<()> {
    auth_service::establecer_pin(&state.pool, &pin)
}

#[tauri::command]
pub fn auth_verify_pin(state: State<AppState>, pin: String) -> AppResult<bool> {
    auth_service::verificar_pin(&state.pool, &pin)
}

#[tauri::command]
pub fn auth_disable_pin(state: State<AppState>, pin_actual: String) -> AppResult<()> {
    auth_service::deshabilitar_pin(&state.pool, &pin_actual)
}

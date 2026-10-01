use tauri::State;

use crate::error::AppResult;
use crate::models::config::{EmpresaConfig, EmpresaConfigActualizar};
use crate::services::config_service;
use crate::state::AppState;

#[tauri::command]
pub fn config_get(state: State<AppState>) -> AppResult<EmpresaConfig> {
    config_service::obtener(&state.pool)
}

#[tauri::command]
pub fn config_update(
    state: State<AppState>,
    datos: EmpresaConfigActualizar,
) -> AppResult<EmpresaConfig> {
    config_service::actualizar(&state.pool, datos)
}

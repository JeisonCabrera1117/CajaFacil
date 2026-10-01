use tauri::State;

use crate::error::AppResult;
use crate::models::devolucion::{Devolucion, DevolucionNueva};
use crate::services::devolucion_service;
use crate::state::AppState;

#[tauri::command]
pub fn devolucion_crear(state: State<AppState>, datos: DevolucionNueva) -> AppResult<Devolucion> {
    devolucion_service::crear(&state.pool, datos)
}

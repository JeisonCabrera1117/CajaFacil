use tauri::State;

use crate::error::AppResult;
use crate::models::cliente::{Cliente, ClienteNuevo};
use crate::services::cliente_service;
use crate::state::AppState;

#[tauri::command]
pub fn cliente_list(state: State<AppState>) -> AppResult<Vec<Cliente>> {
    cliente_service::listar(&state.pool)
}

#[tauri::command]
pub fn cliente_get(state: State<AppState>, id: i64) -> AppResult<Cliente> {
    cliente_service::obtener(&state.pool, id)
}

#[tauri::command]
pub fn cliente_create(state: State<AppState>, datos: ClienteNuevo) -> AppResult<Cliente> {
    cliente_service::crear(&state.pool, datos)
}

#[tauri::command]
pub fn cliente_update(state: State<AppState>, id: i64, datos: ClienteNuevo) -> AppResult<Cliente> {
    cliente_service::actualizar(&state.pool, id, datos)
}

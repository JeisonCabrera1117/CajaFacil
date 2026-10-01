use tauri::State;

use crate::error::AppResult;
use crate::models::categoria::{Categoria, CategoriaNueva};
use crate::services::categoria_service;
use crate::state::AppState;

#[tauri::command]
pub fn categoria_list(state: State<AppState>) -> AppResult<Vec<Categoria>> {
    categoria_service::listar(&state.pool)
}

#[tauri::command]
pub fn categoria_create(state: State<AppState>, datos: CategoriaNueva) -> AppResult<Categoria> {
    categoria_service::crear(&state.pool, datos)
}

#[tauri::command]
pub fn categoria_update(
    state: State<AppState>,
    id: i64,
    datos: CategoriaNueva,
) -> AppResult<Categoria> {
    categoria_service::actualizar(&state.pool, id, datos)
}

#[tauri::command]
pub fn categoria_delete(state: State<AppState>, id: i64) -> AppResult<()> {
    categoria_service::eliminar(&state.pool, id)
}

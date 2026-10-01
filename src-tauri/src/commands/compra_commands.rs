use tauri::State;

use crate::error::AppResult;
use crate::models::compra::{Compra, CompraConDetalle, CompraNueva};
use crate::services::compra_service;
use crate::state::AppState;

#[tauri::command]
pub fn compra_list(state: State<AppState>, proveedor_id: Option<i64>) -> AppResult<Vec<Compra>> {
    compra_service::listar(&state.pool, proveedor_id)
}

#[tauri::command]
pub fn compra_get(state: State<AppState>, id: i64) -> AppResult<CompraConDetalle> {
    compra_service::obtener(&state.pool, id)
}

#[tauri::command]
pub fn compra_create(state: State<AppState>, datos: CompraNueva) -> AppResult<CompraConDetalle> {
    compra_service::crear(&state.pool, datos)
}

#[tauri::command]
pub fn compra_anular(
    state: State<AppState>,
    id: i64,
    motivo: String,
) -> AppResult<CompraConDetalle> {
    compra_service::anular(&state.pool, id, motivo)
}

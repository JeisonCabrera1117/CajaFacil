use tauri::State;

use crate::error::AppResult;
use crate::models::inventario_fisico::{
    InventarioFisico, InventarioFisicoConDetalle, RegistrarConteo,
};
use crate::services::inventario_fisico_service;
use crate::state::AppState;

#[tauri::command]
pub fn inventario_fisico_iniciar(
    state: State<AppState>,
    observaciones: Option<String>,
) -> AppResult<InventarioFisico> {
    inventario_fisico_service::iniciar(&state.pool, observaciones)
}

#[tauri::command]
pub fn inventario_fisico_list(state: State<AppState>) -> AppResult<Vec<InventarioFisico>> {
    inventario_fisico_service::listar(&state.pool)
}

#[tauri::command]
pub fn inventario_fisico_get(
    state: State<AppState>,
    id: i64,
) -> AppResult<InventarioFisicoConDetalle> {
    inventario_fisico_service::obtener(&state.pool, id)
}

#[tauri::command]
pub fn inventario_fisico_registrar_conteo(
    state: State<AppState>,
    datos: RegistrarConteo,
) -> AppResult<InventarioFisicoConDetalle> {
    inventario_fisico_service::registrar_conteo(&state.pool, datos)
}

#[tauri::command]
pub fn inventario_fisico_cerrar(
    state: State<AppState>,
    id: i64,
) -> AppResult<InventarioFisicoConDetalle> {
    inventario_fisico_service::cerrar(&state.pool, id)
}

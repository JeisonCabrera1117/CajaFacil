use tauri::State;

use crate::error::AppResult;
use crate::models::compra::Compra;
use crate::models::proveedor::{Proveedor, ProveedorNuevo};
use crate::services::proveedor_service;
use crate::state::AppState;

#[tauri::command]
pub fn proveedor_list(state: State<AppState>) -> AppResult<Vec<Proveedor>> {
    proveedor_service::listar(&state.pool)
}

#[tauri::command]
pub fn proveedor_get(state: State<AppState>, id: i64) -> AppResult<Proveedor> {
    proveedor_service::obtener(&state.pool, id)
}

#[tauri::command]
pub fn proveedor_create(state: State<AppState>, datos: ProveedorNuevo) -> AppResult<Proveedor> {
    proveedor_service::crear(&state.pool, datos)
}

#[tauri::command]
pub fn proveedor_update(
    state: State<AppState>,
    id: i64,
    datos: ProveedorNuevo,
) -> AppResult<Proveedor> {
    proveedor_service::actualizar(&state.pool, id, datos)
}

#[tauri::command]
pub fn proveedor_delete(state: State<AppState>, id: i64) -> AppResult<()> {
    proveedor_service::eliminar(&state.pool, id)
}

#[tauri::command]
pub fn proveedor_historial_compras(state: State<AppState>, id: i64) -> AppResult<Vec<Compra>> {
    proveedor_service::historial_compras(&state.pool, id)
}

#[tauri::command]
pub fn proveedor_productos_list(state: State<AppState>, id: i64) -> AppResult<Vec<i64>> {
    proveedor_service::productos_que_suministra(&state.pool, id)
}

#[tauri::command]
pub fn proveedor_productos_asignar(
    state: State<AppState>,
    id: i64,
    producto_ids: Vec<i64>,
) -> AppResult<()> {
    proveedor_service::asignar_productos(&state.pool, id, producto_ids)
}

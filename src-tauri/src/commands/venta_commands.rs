use tauri::State;

use crate::error::AppResult;
use crate::models::producto::Producto;
use crate::models::venta::{Venta, VentaConDetalle, VentaFiltro, VentaNueva};
use crate::services::venta_service;
use crate::state::AppState;

#[tauri::command]
pub fn venta_buscar_producto(state: State<AppState>, termino: String) -> AppResult<Vec<Producto>> {
    venta_service::buscar_producto(&state.pool, &termino)
}

#[tauri::command]
pub fn venta_crear(state: State<AppState>, datos: VentaNueva) -> AppResult<VentaConDetalle> {
    venta_service::crear(&state.pool, datos)
}

#[tauri::command]
pub fn venta_anular(state: State<AppState>, id: i64, motivo: String) -> AppResult<VentaConDetalle> {
    venta_service::anular(&state.pool, id, motivo)
}

#[tauri::command]
pub fn venta_list(state: State<AppState>, filtro: VentaFiltro) -> AppResult<Vec<Venta>> {
    venta_service::listar(&state.pool, filtro)
}

#[tauri::command]
pub fn venta_get(state: State<AppState>, id: i64) -> AppResult<VentaConDetalle> {
    venta_service::obtener(&state.pool, id)
}

use tauri::State;

use crate::error::AppResult;
use crate::models::movimiento::MovimientoStock;
use crate::models::producto::{Producto, ProductoFiltro, ProductoNuevo, ProductoPagina};
use crate::services::producto_service;
use crate::services::stock_service::{self, NuevoMovimiento};
use crate::state::AppState;

#[tauri::command]
pub fn producto_list(state: State<AppState>, filtro: ProductoFiltro) -> AppResult<ProductoPagina> {
    producto_service::listar(&state.pool, filtro)
}

#[tauri::command]
pub fn producto_get(state: State<AppState>, id: i64) -> AppResult<Producto> {
    producto_service::obtener(&state.pool, id)
}

#[tauri::command]
pub fn producto_create(state: State<AppState>, datos: ProductoNuevo) -> AppResult<Producto> {
    producto_service::crear(&state.pool, datos)
}

#[tauri::command]
pub fn producto_update(
    state: State<AppState>,
    id: i64,
    datos: ProductoNuevo,
) -> AppResult<Producto> {
    producto_service::actualizar(&state.pool, id, datos)
}

#[tauri::command]
pub fn producto_delete(state: State<AppState>, id: i64) -> AppResult<()> {
    producto_service::eliminar(&state.pool, id)
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AjustarStockEntrada {
    pub producto_id: i64,
    pub tipo: String,
    pub cantidad: i64,
    pub costo_unitario: Option<i64>,
    pub motivo: Option<String>,
}

#[tauri::command]
pub fn producto_ajustar_stock(
    state: State<AppState>,
    datos: AjustarStockEntrada,
) -> AppResult<MovimientoStock> {
    stock_service::registrar_movimiento(
        &state.pool,
        NuevoMovimiento {
            producto_id: datos.producto_id,
            tipo: datos.tipo,
            cantidad: datos.cantidad,
            costo_unitario: datos.costo_unitario,
            motivo: datos.motivo,
            referencia_tipo: Some("manual".into()),
            referencia_id: None,
        },
    )
}

#[tauri::command]
pub fn kardex_get(state: State<AppState>, producto_id: i64) -> AppResult<Vec<MovimientoStock>> {
    stock_service::kardex(&state.pool, producto_id)
}

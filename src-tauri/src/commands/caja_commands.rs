use tauri::State;

use crate::error::AppResult;
use crate::models::caja::CajaSesion;
use crate::services::caja_service;
use crate::state::AppState;

#[tauri::command]
pub fn caja_estado_actual(state: State<AppState>) -> AppResult<Option<CajaSesion>> {
    caja_service::estado_actual(&state.pool)
}

#[tauri::command]
pub fn caja_abrir(state: State<AppState>, monto_apertura: i64) -> AppResult<CajaSesion> {
    caja_service::abrir(&state.pool, monto_apertura)
}

#[tauri::command]
pub fn caja_cerrar(
    state: State<AppState>,
    monto_cierre_contado: i64,
    observaciones: Option<String>,
) -> AppResult<CajaSesion> {
    caja_service::cerrar(&state.pool, monto_cierre_contado, observaciones)
}

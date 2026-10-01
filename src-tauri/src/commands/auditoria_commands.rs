use tauri::State;

use crate::error::AppResult;
use crate::services::audit_service::{self, EntradaAuditoria, FiltroAuditoria};
use crate::state::AppState;

#[tauri::command]
pub fn auditoria_list(
    state: State<AppState>,
    filtro: FiltroAuditoria,
) -> AppResult<Vec<EntradaAuditoria>> {
    audit_service::listar(&state.pool, filtro)
}

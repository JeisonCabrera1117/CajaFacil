use tauri::State;

use crate::error::AppResult;
use crate::models::dashboard::{DashboardGraficas, DashboardIndicadores};
use crate::services::dashboard_service;
use crate::state::AppState;

#[tauri::command]
pub fn dashboard_indicadores(state: State<AppState>) -> AppResult<DashboardIndicadores> {
    dashboard_service::indicadores(&state.pool)
}

#[tauri::command]
pub fn dashboard_graficas(
    state: State<AppState>,
    desde: Option<String>,
    hasta: Option<String>,
) -> AppResult<DashboardGraficas> {
    let hasta = hasta.unwrap_or_else(|| chrono::Local::now().format("%Y-%m-%d").to_string());
    let desde = desde.unwrap_or_else(|| {
        (chrono::Local::now() - chrono::Duration::days(29))
            .format("%Y-%m-%d")
            .to_string()
    });
    dashboard_service::graficas(&state.pool, &desde, &hasta)
}

use tauri::{Manager, State};

use crate::error::AppResult;
use crate::services::respaldo_service::{self, Respaldo};
use crate::state::AppState;

#[tauri::command]
pub fn respaldo_list(state: State<AppState>) -> AppResult<Vec<Respaldo>> {
    respaldo_service::listar(&state.pool)
}

#[tauri::command]
pub fn respaldo_crear(state: State<AppState>) -> AppResult<Respaldo> {
    respaldo_service::crear(&state.pool, &state.respaldos_dir, "manual")
}

#[tauri::command]
pub fn respaldo_eliminar(state: State<AppState>, id: i64) -> AppResult<()> {
    respaldo_service::eliminar(&state.pool, id)
}

/// Deja listo el marcador de restauración y cierra la aplicación: el archivo
/// de base de datos está en uso mientras la app corre, así que la restauración
/// real ocurre al arrancar de nuevo (ver `lib.rs`).
#[tauri::command]
pub fn respaldo_restaurar(app: tauri::AppHandle, state: State<AppState>, id: i64) -> AppResult<()> {
    let app_data_dir = app.path().app_data_dir().map_err(|e| {
        crate::error::AppError::Interno(format!("No se pudo resolver el directorio de datos: {e}"))
    })?;
    respaldo_service::solicitar_restauracion(&state.pool, &app_data_dir, id)?;
    app.exit(0);
    Ok(())
}

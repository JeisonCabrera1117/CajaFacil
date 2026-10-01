use tauri::State;

use crate::error::AppResult;
use crate::models::importacion::PlantillaGenerada;
use crate::models::reporte::{
    FilaComprasProveedor, FilaInventarioValorizado, FilaReporteVenta, FilaRotacion, FilaUtilidad,
    FiltroReporteVentas, SolicitudExportacion,
};
use crate::services::reporte_service;
use crate::state::AppState;

#[tauri::command]
pub fn reporte_ventas(
    state: State<AppState>,
    filtro: FiltroReporteVentas,
) -> AppResult<Vec<FilaReporteVenta>> {
    reporte_service::ventas(&state.pool, &filtro)
}

#[tauri::command]
pub fn reporte_utilidad(
    state: State<AppState>,
    desde: String,
    hasta: String,
) -> AppResult<Vec<FilaUtilidad>> {
    reporte_service::utilidad(&state.pool, &desde, &hasta)
}

#[tauri::command]
pub fn reporte_inventario_valorizado(
    state: State<AppState>,
) -> AppResult<Vec<FilaInventarioValorizado>> {
    reporte_service::inventario_valorizado(&state.pool)
}

#[tauri::command]
pub fn reporte_rotacion(
    state: State<AppState>,
    desde: String,
    hasta: String,
) -> AppResult<Vec<FilaRotacion>> {
    reporte_service::rotacion(&state.pool, &desde, &hasta)
}

#[tauri::command]
pub fn reporte_compras_proveedor(
    state: State<AppState>,
    desde: String,
    hasta: String,
) -> AppResult<Vec<FilaComprasProveedor>> {
    reporte_service::compras_por_proveedor(&state.pool, &desde, &hasta)
}

#[tauri::command]
pub fn reporte_exportar(solicitud: SolicitudExportacion) -> AppResult<PlantillaGenerada> {
    crate::exportacion::generar(
        &solicitud.titulo,
        &solicitud.columnas,
        &solicitud.filas,
        &solicitud.formato,
    )
}

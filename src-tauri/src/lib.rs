mod commands;
mod db;
mod error;
mod exportacion;
mod importacion;
mod models;
mod pdf;
mod services;
mod state;

use tauri::Manager;

use state::AppState;

fn init_logging(logs_dir: &std::path::Path) {
    use tracing_appender::rolling;
    use tracing_subscriber::{fmt, EnvFilter};

    std::fs::create_dir_all(logs_dir).ok();
    let file_appender = rolling::daily(logs_dir, "cajafacil.log");
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);
    // El guard debe vivir mientras la app corra para no perder líneas de log en el buffer.
    Box::leak(Box::new(guard));

    fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .with_writer(non_blocking)
        .with_ansi(false)
        .init();
}

/// Si el usuario pidió restaurar un respaldo, `respaldo_commands::respaldo_restaurar`
/// dejó un marcador con la ruta del archivo elegido y cerró la app (el archivo de
/// base de datos activo no se puede reemplazar mientras sigue abierto). Al volver a
/// arrancar, antes de abrir el pool, se copia ese respaldo sobre la base de datos real.
fn aplicar_restauracion_pendiente(app_data_dir: &std::path::Path, db_path: &std::path::Path) {
    let marcador = app_data_dir.join("restaurar.marcador");
    let Ok(origen) = std::fs::read_to_string(&marcador) else {
        return;
    };
    let origen = std::path::PathBuf::from(origen.trim());
    if origen.is_file() {
        if let Err(e) = std::fs::copy(&origen, db_path) {
            tracing::error!("No se pudo restaurar el respaldo {:?}: {e}", origen);
        } else {
            tracing::info!("Base de datos restaurada desde {:?}", origen);
        }
        std::fs::remove_file(db_path.with_extension("sqlite3-wal")).ok();
        std::fs::remove_file(db_path.with_extension("sqlite3-shm")).ok();
    }
    std::fs::remove_file(&marcador).ok();
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .setup(|app| {
            let app_data_dir = app
                .path()
                .app_data_dir()
                .expect("no se pudo resolver el directorio de datos de la aplicación");
            std::fs::create_dir_all(&app_data_dir)
                .expect("no se pudo crear el directorio de datos");

            init_logging(&app_data_dir.join("logs"));
            tracing::info!("Iniciando CajaFácil. Datos en {:?}", app_data_dir);

            let db_path = app_data_dir.join("cajafacil.sqlite3");
            aplicar_restauracion_pendiente(&app_data_dir, &db_path);

            let pool = db::init_pool(&db_path).expect("no se pudo inicializar la base de datos");

            let comprobantes_dir = app_data_dir.join("comprobantes");
            let importaciones_dir = app_data_dir.join("importaciones");
            let respaldos_dir = app_data_dir.join("respaldos");

            app.manage(AppState {
                pool,
                comprobantes_dir,
                importaciones_dir,
                respaldos_dir,
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::auth_commands::auth_check_required,
            commands::auth_commands::auth_set_pin,
            commands::auth_commands::auth_verify_pin,
            commands::auth_commands::auth_disable_pin,
            commands::config_commands::config_get,
            commands::config_commands::config_update,
            commands::categoria_commands::categoria_list,
            commands::categoria_commands::categoria_create,
            commands::categoria_commands::categoria_update,
            commands::categoria_commands::categoria_delete,
            commands::proveedor_commands::proveedor_list,
            commands::proveedor_commands::proveedor_get,
            commands::proveedor_commands::proveedor_create,
            commands::proveedor_commands::proveedor_update,
            commands::proveedor_commands::proveedor_delete,
            commands::proveedor_commands::proveedor_historial_compras,
            commands::proveedor_commands::proveedor_productos_list,
            commands::proveedor_commands::proveedor_productos_asignar,
            commands::producto_commands::producto_list,
            commands::producto_commands::producto_get,
            commands::producto_commands::producto_create,
            commands::producto_commands::producto_update,
            commands::producto_commands::producto_delete,
            commands::producto_commands::producto_ajustar_stock,
            commands::producto_commands::kardex_get,
            commands::compra_commands::compra_list,
            commands::compra_commands::compra_get,
            commands::compra_commands::compra_create,
            commands::compra_commands::compra_anular,
            commands::inventario_commands::inventario_fisico_iniciar,
            commands::inventario_commands::inventario_fisico_list,
            commands::inventario_commands::inventario_fisico_get,
            commands::inventario_commands::inventario_fisico_registrar_conteo,
            commands::inventario_commands::inventario_fisico_cerrar,
            commands::cliente_commands::cliente_list,
            commands::cliente_commands::cliente_get,
            commands::cliente_commands::cliente_create,
            commands::cliente_commands::cliente_update,
            commands::caja_commands::caja_estado_actual,
            commands::caja_commands::caja_abrir,
            commands::caja_commands::caja_cerrar,
            commands::venta_commands::venta_buscar_producto,
            commands::venta_commands::venta_crear,
            commands::venta_commands::venta_anular,
            commands::venta_commands::venta_list,
            commands::venta_commands::venta_get,
            commands::devolucion_commands::devolucion_crear,
            commands::comprobante_commands::comprobante_generar,
            commands::comprobante_commands::comprobante_regenerar,
            commands::comprobante_commands::comprobante_abrir,
            commands::comprobante_commands::comprobante_copiar_imagen,
            commands::importacion_commands::import_entidades,
            commands::importacion_commands::import_campos,
            commands::importacion_commands::import_plantilla_descargar,
            commands::importacion_commands::import_detectar_archivo,
            commands::importacion_commands::import_preview,
            commands::importacion_commands::import_ejecutar,
            commands::importacion_commands::import_historial,
            commands::dashboard_commands::dashboard_indicadores,
            commands::dashboard_commands::dashboard_graficas,
            commands::reporte_commands::reporte_ventas,
            commands::reporte_commands::reporte_utilidad,
            commands::reporte_commands::reporte_inventario_valorizado,
            commands::reporte_commands::reporte_rotacion,
            commands::reporte_commands::reporte_compras_proveedor,
            commands::reporte_commands::reporte_exportar,
            commands::respaldo_commands::respaldo_list,
            commands::respaldo_commands::respaldo_crear,
            commands::respaldo_commands::respaldo_eliminar,
            commands::respaldo_commands::respaldo_restaurar,
            commands::auditoria_commands::auditoria_list,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

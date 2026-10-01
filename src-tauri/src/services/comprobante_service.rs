use std::fs;
use std::path::Path;

use rusqlite::params;

use crate::db::DbPool;
use crate::error::{AppError, AppResult};
use crate::models::comprobante::Comprobante;
use crate::pdf::{self, datos, elementos};
use crate::services::audit_service;

const TIPOS_VALIDOS: [&str; 3] = ["carta", "termico80", "imagen"];

pub fn generar(
    pool: &DbPool,
    comprobantes_dir: &Path,
    venta_id: i64,
    tipo: &str,
) -> AppResult<Comprobante> {
    if !TIPOS_VALIDOS.contains(&tipo) {
        return Err(AppError::Validacion("Tipo de comprobante inválido.".into()));
    }

    let datos_comprobante = datos::construir(pool, venta_id)?;
    let elementos = elementos::construir(&datos_comprobante);
    let bytes = pdf::generar_bytes(&elementos, datos_comprobante.logo_path.as_deref(), tipo)?;

    let ruta = pdf::ruta_para(
        comprobantes_dir,
        &datos_comprobante.venta.venta.numero_comprobante,
        tipo,
        &datos_comprobante.venta.venta.fecha,
    )?;
    if let Some(padre) = ruta.parent() {
        fs::create_dir_all(padre).map_err(|e| {
            AppError::Interno(format!("No se pudo crear la carpeta de comprobantes: {e}"))
        })?;
    }
    fs::write(&ruta, &bytes)
        .map_err(|e| AppError::Interno(format!("No se pudo guardar el comprobante: {e}")))?;

    let ruta_texto = ruta.to_string_lossy().to_string();
    let conn = pool.get()?;
    conn.execute(
        "INSERT INTO comprobantes (venta_id, tipo, ruta_archivo) VALUES (?1, ?2, ?3)",
        params![venta_id, tipo, ruta_texto],
    )?;
    let id = conn.last_insert_rowid();
    let generado_en: String = conn.query_row(
        "SELECT generado_en FROM comprobantes WHERE id = ?1",
        [id],
        |r| r.get(0),
    )?;
    drop(conn);

    let creado = Comprobante {
        id,
        venta_id,
        tipo: tipo.to_string(),
        ruta_archivo: ruta_texto,
        generado_en,
    };
    audit_service::registrar(pool, "comprobante", Some(id), "crear", &creado)?;
    Ok(creado)
}

pub fn abrir(ruta_archivo: &str) -> AppResult<()> {
    tauri_plugin_opener::open_path(ruta_archivo, None::<&str>)
        .map_err(|e| AppError::Interno(format!("No se pudo abrir el comprobante: {e}")))
}

pub fn copiar_imagen(ruta_archivo: &str) -> AppResult<()> {
    if !ruta_archivo.to_lowercase().ends_with(".png") {
        return Err(AppError::Validacion(
            "Solo se puede copiar al portapapeles un comprobante de tipo imagen.".into(),
        ));
    }
    let bytes = fs::read(ruta_archivo)
        .map_err(|e| AppError::Interno(format!("No se pudo leer el comprobante: {e}")))?;
    let imagen = image::load_from_memory(&bytes)
        .map_err(|e| AppError::Interno(format!("No se pudo decodificar la imagen: {e}")))?
        .to_rgba8();
    let (ancho, alto) = imagen.dimensions();

    let datos_imagen = arboard::ImageData {
        width: ancho as usize,
        height: alto as usize,
        bytes: std::borrow::Cow::Owned(imagen.into_raw()),
    };
    let mut portapapeles = arboard::Clipboard::new()
        .map_err(|e| AppError::Interno(format!("No se pudo acceder al portapapeles: {e}")))?;
    portapapeles.set_image(datos_imagen).map_err(|e| {
        AppError::Interno(format!("No se pudo copiar la imagen al portapapeles: {e}"))
    })?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::venta::{VentaItemNuevo, VentaNueva};
    use crate::services::venta_service;
    use tempfile::tempdir;

    fn pool_de_prueba() -> DbPool {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.sqlite3");
        let pool = crate::db::init_pool(&path).unwrap();
        std::mem::forget(dir);
        pool
    }

    fn crear_venta_de_prueba(pool: &DbPool) -> i64 {
        let conn = pool.get().unwrap();
        conn.execute(
            "INSERT INTO productos (sku, nombre, unidad_medida, precio_costo, precio_venta, stock_actual)
             VALUES ('SKU-PDF-1', 'Producto de prueba', 'unidad', 1000, 5000, 10)",
            [],
        )
        .unwrap();
        let producto_id = conn.last_insert_rowid();
        drop(conn);

        venta_service::crear(
            pool,
            VentaNueva {
                cliente_id: None,
                metodo_pago: "efectivo".into(),
                valor_recibido: Some(15_000),
                descuento_global: 0,
                items: vec![VentaItemNuevo {
                    producto_id,
                    cantidad: 2,
                    precio_unitario: 5000,
                    descuento_pct: 0.0,
                    descuento_valor: 0,
                    impuesto_pct: 19.0,
                }],
            },
        )
        .unwrap()
        .venta
        .id
    }

    #[test]
    fn genera_los_tres_formatos_y_los_registra_en_la_bd() {
        let pool = pool_de_prueba();
        let venta_id = crear_venta_de_prueba(&pool);
        let dir_comprobantes = tempdir().unwrap();

        for (tipo, extension) in [("carta", "pdf"), ("termico80", "pdf"), ("imagen", "png")] {
            let comprobante = generar(&pool, dir_comprobantes.path(), venta_id, tipo).unwrap();
            assert_eq!(comprobante.tipo, tipo);

            let ruta = std::path::Path::new(&comprobante.ruta_archivo);
            assert!(
                ruta.exists(),
                "el archivo del comprobante {tipo} debería existir"
            );
            assert_eq!(ruta.extension().unwrap(), extension);
            assert!(
                fs::metadata(ruta).unwrap().len() > 0,
                "el comprobante {tipo} no debería estar vacío"
            );

            // Organizado en carpetas año/mes.
            assert!(ruta
                .parent()
                .unwrap()
                .parent()
                .unwrap()
                .starts_with(dir_comprobantes.path()));
        }
    }

    #[test]
    fn regenerar_crea_un_registro_nuevo_sin_duplicar_el_primero() {
        let pool = pool_de_prueba();
        let venta_id = crear_venta_de_prueba(&pool);
        let dir_comprobantes = tempdir().unwrap();

        let primero = generar(&pool, dir_comprobantes.path(), venta_id, "carta").unwrap();
        let segundo = generar(&pool, dir_comprobantes.path(), venta_id, "carta").unwrap();
        assert_ne!(primero.id, segundo.id);

        let conn = pool.get().unwrap();
        let total: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM comprobantes WHERE venta_id = ?1",
                [venta_id],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(total, 2);
    }

    #[test]
    fn rechaza_tipo_invalido() {
        let pool = pool_de_prueba();
        let venta_id = crear_venta_de_prueba(&pool);
        let dir_comprobantes = tempdir().unwrap();
        assert!(generar(&pool, dir_comprobantes.path(), venta_id, "excel").is_err());
    }
}

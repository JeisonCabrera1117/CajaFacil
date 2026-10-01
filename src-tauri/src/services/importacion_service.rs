use std::collections::HashMap;
use std::fs;
use std::path::Path;

use rusqlite::params;

use crate::db::DbPool;
use crate::error::{AppError, AppResult};
use crate::importacion;
use crate::models::importacion::{
    CampoImport, DeteccionArchivo, EntidadImportable, Importacion, ModoImport, PlantillaGenerada,
    ResumenImportacion,
};
use crate::services::audit_service;

pub fn entidades_importables() -> Vec<EntidadImportable> {
    importacion::entidades_importables()
}

pub fn campos_de(entidad: &str) -> AppResult<Vec<CampoImport>> {
    importacion::campos_de(entidad)
}

pub fn detectar(
    ruta: &str,
    hoja: Option<&str>,
    separador: Option<char>,
) -> AppResult<DeteccionArchivo> {
    importacion::detectar(ruta, hoja, separador)
}

pub fn plantilla(entidad: &str, formato: &str) -> AppResult<PlantillaGenerada> {
    importacion::plantillas::generar(entidad, formato)
}

pub fn previsualizar(
    pool: &DbPool,
    ruta: &str,
    hoja: Option<&str>,
    separador: Option<char>,
    entidad: &str,
    mapeo: &HashMap<String, String>,
    modo: ModoImport,
) -> AppResult<ResumenImportacion> {
    importacion::procesar(
        pool,
        ruta,
        hoja,
        separador,
        entidad,
        mapeo,
        modo,
        false,
        |_, _| {},
    )
}

#[allow(clippy::too_many_arguments)]
pub fn ejecutar(
    pool: &DbPool,
    importaciones_dir: &Path,
    ruta: &str,
    hoja: Option<&str>,
    separador: Option<char>,
    entidad: &str,
    mapeo: &HashMap<String, String>,
    modo: ModoImport,
    archivo_nombre_original: &str,
    mut sobre_progreso: impl FnMut(i64, i64),
) -> AppResult<Importacion> {
    let resumen = importacion::procesar(
        pool,
        ruta,
        hoja,
        separador,
        entidad,
        mapeo,
        modo,
        true,
        &mut sobre_progreso,
    )?;

    let archivo_rechazados_path = if resumen.filas_rechazadas.is_empty() {
        None
    } else {
        Some(guardar_rechazados(importaciones_dir, entidad, &resumen)?)
    };

    let modo_texto = match modo {
        ModoImport::Crear => "crear",
        ModoImport::Actualizar => "actualizar",
        ModoImport::CrearYActualizar => "crear_y_actualizar",
    };

    let conn = pool.get()?;
    conn.execute(
        "INSERT INTO importaciones (entidad, archivo_nombre, modo, total_filas, creados, actualizados,
            omitidos, con_error, archivo_rechazados_path)
         VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9)",
        params![
            entidad,
            archivo_nombre_original,
            modo_texto,
            resumen.total_filas,
            resumen.creados,
            resumen.actualizados,
            resumen.omitidos,
            resumen.con_error,
            archivo_rechazados_path,
        ],
    )?;
    let id = conn.last_insert_rowid();
    let fecha: String =
        conn.query_row("SELECT fecha FROM importaciones WHERE id = ?1", [id], |r| {
            r.get(0)
        })?;
    drop(conn);

    let registro = Importacion {
        id,
        entidad: entidad.to_string(),
        archivo_nombre: archivo_nombre_original.to_string(),
        fecha,
        modo: modo_texto.to_string(),
        total_filas: resumen.total_filas,
        creados: resumen.creados,
        actualizados: resumen.actualizados,
        omitidos: resumen.omitidos,
        con_error: resumen.con_error,
        archivo_rechazados_path,
    };
    audit_service::registrar(pool, "importacion", Some(id), "crear", &registro)?;
    Ok(registro)
}

pub fn historial(pool: &DbPool) -> AppResult<Vec<Importacion>> {
    let conn = pool.get()?;
    let mut stmt = conn.prepare(
        "SELECT id, entidad, archivo_nombre, fecha, modo, total_filas, creados, actualizados, omitidos,
                con_error, archivo_rechazados_path
         FROM importaciones ORDER BY fecha DESC",
    )?;
    let filas = stmt.query_map([], |r| {
        Ok(Importacion {
            id: r.get(0)?,
            entidad: r.get(1)?,
            archivo_nombre: r.get(2)?,
            fecha: r.get(3)?,
            modo: r.get(4)?,
            total_filas: r.get(5)?,
            creados: r.get(6)?,
            actualizados: r.get(7)?,
            omitidos: r.get(8)?,
            con_error: r.get(9)?,
            archivo_rechazados_path: r.get(10)?,
        })
    })?;
    let mut out = Vec::new();
    for f in filas {
        out.push(f?);
    }
    Ok(out)
}

fn guardar_rechazados(
    importaciones_dir: &Path,
    entidad: &str,
    resumen: &ResumenImportacion,
) -> AppResult<String> {
    fs::create_dir_all(importaciones_dir).map_err(|e| {
        AppError::Interno(format!("No se pudo crear la carpeta de importaciones: {e}"))
    })?;
    let nombre = format!(
        "rechazados-{entidad}-{}.csv",
        chrono::Local::now().format("%Y%m%d-%H%M%S")
    );
    let ruta = importaciones_dir.join(nombre);

    let mut contenido = String::from("fila,motivo\r\n");
    for f in &resumen.filas_rechazadas {
        contenido.push_str(&format!(
            "{},\"{}\"\r\n",
            f.fila,
            f.motivo.replace('"', "'")
        ));
    }
    fs::write(&ruta, contenido).map_err(|e| {
        AppError::Interno(format!("No se pudo guardar el archivo de rechazados: {e}"))
    })?;
    Ok(ruta.to_string_lossy().to_string())
}

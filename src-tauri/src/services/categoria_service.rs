use rusqlite::{params, OptionalExtension};

use crate::db::DbPool;
use crate::error::{AppError, AppResult};
use crate::models::categoria::{Categoria, CategoriaNueva};
use crate::services::audit_service;

fn map(row: &rusqlite::Row) -> rusqlite::Result<Categoria> {
    Ok(Categoria {
        id: row.get(0)?,
        nombre: row.get(1)?,
        categoria_padre_id: row.get(2)?,
    })
}

pub fn listar(pool: &DbPool) -> AppResult<Vec<Categoria>> {
    let conn = pool.get()?;
    let mut stmt =
        conn.prepare("SELECT id, nombre, categoria_padre_id FROM categorias ORDER BY nombre")?;
    let rows = stmt.query_map([], map)?;
    let mut out = Vec::new();
    for r in rows {
        out.push(r?);
    }
    Ok(out)
}

fn obtener(pool: &DbPool, id: i64) -> AppResult<Categoria> {
    let conn = pool.get()?;
    conn.query_row(
        "SELECT id, nombre, categoria_padre_id FROM categorias WHERE id = ?1",
        [id],
        map,
    )
    .optional()?
    .ok_or_else(|| AppError::NoEncontrado("La categoría no existe.".into()))
}

fn validar(datos: &CategoriaNueva, id_actual: Option<i64>) -> AppResult<()> {
    if datos.nombre.trim().is_empty() {
        return Err(AppError::Validacion(
            "El nombre de la categoría es obligatorio.".into(),
        ));
    }
    if let (Some(padre), Some(actual)) = (datos.categoria_padre_id, id_actual) {
        if padre == actual {
            return Err(AppError::Validacion(
                "Una categoría no puede ser su propia categoría padre.".into(),
            ));
        }
    }
    Ok(())
}

pub fn crear(pool: &DbPool, datos: CategoriaNueva) -> AppResult<Categoria> {
    validar(&datos, None)?;
    let conn = pool.get()?;
    conn.execute(
        "INSERT INTO categorias (nombre, categoria_padre_id) VALUES (?1, ?2)",
        params![datos.nombre, datos.categoria_padre_id],
    )?;
    let id = conn.last_insert_rowid();
    drop(conn);
    let creada = obtener(pool, id)?;
    audit_service::registrar(pool, "categoria", Some(id), "crear", &creada)?;
    Ok(creada)
}

pub fn actualizar(pool: &DbPool, id: i64, datos: CategoriaNueva) -> AppResult<Categoria> {
    validar(&datos, Some(id))?;
    let conn = pool.get()?;
    let filas = conn.execute(
        "UPDATE categorias SET nombre = ?1, categoria_padre_id = ?2 WHERE id = ?3",
        params![datos.nombre, datos.categoria_padre_id, id],
    )?;
    if filas == 0 {
        return Err(AppError::NoEncontrado("La categoría no existe.".into()));
    }
    drop(conn);
    let actualizada = obtener(pool, id)?;
    audit_service::registrar(pool, "categoria", Some(id), "modificar", &actualizada)?;
    Ok(actualizada)
}

pub fn eliminar(pool: &DbPool, id: i64) -> AppResult<()> {
    let conn = pool.get()?;
    let filas = conn.execute("DELETE FROM categorias WHERE id = ?1", [id])?;
    if filas == 0 {
        return Err(AppError::NoEncontrado("La categoría no existe.".into()));
    }
    drop(conn);
    audit_service::registrar(pool, "categoria", Some(id), "eliminar", &id)?;
    Ok(())
}

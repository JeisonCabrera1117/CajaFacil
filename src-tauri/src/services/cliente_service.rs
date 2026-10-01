use rusqlite::{params, OptionalExtension};

use crate::db::DbPool;
use crate::error::{AppError, AppResult};
use crate::models::cliente::{Cliente, ClienteNuevo};
use crate::services::audit_service;

const COLUMNAS: &str = "id, documento, nombre, telefono, correo, direccion";

fn map(row: &rusqlite::Row) -> rusqlite::Result<Cliente> {
    Ok(Cliente {
        id: row.get(0)?,
        documento: row.get(1)?,
        nombre: row.get(2)?,
        telefono: row.get(3)?,
        correo: row.get(4)?,
        direccion: row.get(5)?,
    })
}

pub fn listar(pool: &DbPool) -> AppResult<Vec<Cliente>> {
    let conn = pool.get()?;
    let sql = format!("SELECT {COLUMNAS} FROM clientes ORDER BY nombre");
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map([], map)?;
    let mut out = Vec::new();
    for r in rows {
        out.push(r?);
    }
    Ok(out)
}

pub fn obtener(pool: &DbPool, id: i64) -> AppResult<Cliente> {
    let conn = pool.get()?;
    let sql = format!("SELECT {COLUMNAS} FROM clientes WHERE id = ?1");
    conn.query_row(&sql, [id], map)
        .optional()?
        .ok_or_else(|| AppError::NoEncontrado("El cliente no existe.".into()))
}

fn validar(datos: &ClienteNuevo) -> AppResult<()> {
    if datos.nombre.trim().is_empty() {
        return Err(AppError::Validacion(
            "El nombre del cliente es obligatorio.".into(),
        ));
    }
    Ok(())
}

pub fn crear(pool: &DbPool, datos: ClienteNuevo) -> AppResult<Cliente> {
    validar(&datos)?;
    let conn = pool.get()?;
    conn.execute(
        "INSERT INTO clientes (documento, nombre, telefono, correo, direccion) VALUES (?1,?2,?3,?4,?5)",
        params![datos.documento, datos.nombre, datos.telefono, datos.correo, datos.direccion],
    )?;
    let id = conn.last_insert_rowid();
    drop(conn);
    let creado = obtener(pool, id)?;
    audit_service::registrar(pool, "cliente", Some(id), "crear", &creado)?;
    Ok(creado)
}

pub fn actualizar(pool: &DbPool, id: i64, datos: ClienteNuevo) -> AppResult<Cliente> {
    validar(&datos)?;
    let conn = pool.get()?;
    let filas = conn.execute(
        "UPDATE clientes SET documento=?1, nombre=?2, telefono=?3, correo=?4, direccion=?5 WHERE id=?6",
        params![datos.documento, datos.nombre, datos.telefono, datos.correo, datos.direccion, id],
    )?;
    if filas == 0 {
        return Err(AppError::NoEncontrado("El cliente no existe.".into()));
    }
    drop(conn);
    let actualizado = obtener(pool, id)?;
    audit_service::registrar(pool, "cliente", Some(id), "modificar", &actualizado)?;
    Ok(actualizado)
}

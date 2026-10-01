use rusqlite::{params, OptionalExtension};

use crate::db::DbPool;
use crate::error::{AppError, AppResult};
use crate::models::compra::Compra;
use crate::models::proveedor::{Proveedor, ProveedorNuevo};
use crate::services::audit_service;

const COLUMNAS: &str =
    "id, nit, razon_social, contacto, telefono, correo, direccion, condiciones_pago, activo";

fn map(row: &rusqlite::Row) -> rusqlite::Result<Proveedor> {
    Ok(Proveedor {
        id: row.get(0)?,
        nit: row.get(1)?,
        razon_social: row.get(2)?,
        contacto: row.get(3)?,
        telefono: row.get(4)?,
        correo: row.get(5)?,
        direccion: row.get(6)?,
        condiciones_pago: row.get(7)?,
        activo: row.get::<_, i64>(8)? != 0,
    })
}

fn map_compra(row: &rusqlite::Row) -> rusqlite::Result<Compra> {
    Ok(Compra {
        id: row.get(0)?,
        proveedor_id: row.get(1)?,
        proveedor_nombre: row.get(2)?,
        numero: row.get(3)?,
        fecha: row.get(4)?,
        subtotal: row.get(5)?,
        impuestos: row.get(6)?,
        total: row.get(7)?,
        estado: row.get(8)?,
        observaciones: row.get(9)?,
    })
}

pub fn listar(pool: &DbPool) -> AppResult<Vec<Proveedor>> {
    let conn = pool.get()?;
    let sql = format!("SELECT {COLUMNAS} FROM proveedores ORDER BY razon_social");
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map([], map)?;
    let mut out = Vec::new();
    for r in rows {
        out.push(r?);
    }
    Ok(out)
}

pub fn obtener(pool: &DbPool, id: i64) -> AppResult<Proveedor> {
    let conn = pool.get()?;
    let sql = format!("SELECT {COLUMNAS} FROM proveedores WHERE id = ?1");
    conn.query_row(&sql, [id], map)
        .optional()?
        .ok_or_else(|| AppError::NoEncontrado("El proveedor no existe.".into()))
}

fn validar(datos: &ProveedorNuevo) -> AppResult<()> {
    if datos.razon_social.trim().is_empty() {
        return Err(AppError::Validacion(
            "La razón social es obligatoria.".into(),
        ));
    }
    Ok(())
}

pub fn crear(pool: &DbPool, datos: ProveedorNuevo) -> AppResult<Proveedor> {
    validar(&datos)?;
    let conn = pool.get()?;
    conn.execute(
        "INSERT INTO proveedores (nit, razon_social, contacto, telefono, correo, direccion, condiciones_pago, activo)
         VALUES (?1,?2,?3,?4,?5,?6,?7,?8)",
        params![
            datos.nit, datos.razon_social, datos.contacto, datos.telefono, datos.correo,
            datos.direccion, datos.condiciones_pago, datos.activo as i64,
        ],
    )?;
    let id = conn.last_insert_rowid();
    drop(conn);
    let creado = obtener(pool, id)?;
    audit_service::registrar(pool, "proveedor", Some(id), "crear", &creado)?;
    Ok(creado)
}

pub fn actualizar(pool: &DbPool, id: i64, datos: ProveedorNuevo) -> AppResult<Proveedor> {
    validar(&datos)?;
    let conn = pool.get()?;
    let filas = conn.execute(
        "UPDATE proveedores SET nit=?1, razon_social=?2, contacto=?3, telefono=?4, correo=?5,
            direccion=?6, condiciones_pago=?7, activo=?8 WHERE id = ?9",
        params![
            datos.nit,
            datos.razon_social,
            datos.contacto,
            datos.telefono,
            datos.correo,
            datos.direccion,
            datos.condiciones_pago,
            datos.activo as i64,
            id,
        ],
    )?;
    if filas == 0 {
        return Err(AppError::NoEncontrado("El proveedor no existe.".into()));
    }
    drop(conn);
    let actualizado = obtener(pool, id)?;
    audit_service::registrar(pool, "proveedor", Some(id), "modificar", &actualizado)?;
    Ok(actualizado)
}

pub fn eliminar(pool: &DbPool, id: i64) -> AppResult<()> {
    let conn = pool.get()?;
    let filas = conn.execute("DELETE FROM proveedores WHERE id = ?1", [id])?;
    if filas == 0 {
        return Err(AppError::NoEncontrado("El proveedor no existe.".into()));
    }
    drop(conn);
    audit_service::registrar(pool, "proveedor", Some(id), "eliminar", &id)?;
    Ok(())
}

pub fn historial_compras(pool: &DbPool, proveedor_id: i64) -> AppResult<Vec<Compra>> {
    let conn = pool.get()?;
    let mut stmt = conn.prepare(
        "SELECT c.id, c.proveedor_id, pr.razon_social, c.numero, c.fecha, c.subtotal,
                c.impuestos, c.total, c.estado, c.observaciones
         FROM compras c LEFT JOIN proveedores pr ON pr.id = c.proveedor_id
         WHERE c.proveedor_id = ?1 ORDER BY c.fecha DESC",
    )?;
    let rows = stmt.query_map([proveedor_id], map_compra)?;
    let mut out = Vec::new();
    for r in rows {
        out.push(r?);
    }
    Ok(out)
}

pub fn productos_que_suministra(pool: &DbPool, proveedor_id: i64) -> AppResult<Vec<i64>> {
    let conn = pool.get()?;
    let mut stmt =
        conn.prepare("SELECT producto_id FROM proveedor_productos WHERE proveedor_id = ?1")?;
    let rows = stmt.query_map([proveedor_id], |r| r.get::<_, i64>(0))?;
    let mut out = Vec::new();
    for r in rows {
        out.push(r?);
    }
    Ok(out)
}

pub fn asignar_productos(
    pool: &DbPool,
    proveedor_id: i64,
    producto_ids: Vec<i64>,
) -> AppResult<()> {
    let mut conn = pool.get()?;
    let tx = conn.transaction()?;
    tx.execute(
        "DELETE FROM proveedor_productos WHERE proveedor_id = ?1",
        [proveedor_id],
    )?;
    for producto_id in producto_ids {
        tx.execute(
            "INSERT INTO proveedor_productos (proveedor_id, producto_id) VALUES (?1, ?2)",
            params![proveedor_id, producto_id],
        )?;
    }
    tx.commit()?;
    Ok(())
}

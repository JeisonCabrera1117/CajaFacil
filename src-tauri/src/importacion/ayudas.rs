use std::collections::HashMap;

use rusqlite::{Connection, OptionalExtension};

pub fn texto(fila: &HashMap<String, String>, campo: &str) -> Option<String> {
    fila.get(campo)
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

pub fn traducir_error_sql(e: &rusqlite::Error) -> String {
    if let rusqlite::Error::SqliteFailure(_, Some(msg)) = e {
        format!("Error de base de datos: {msg}")
    } else {
        format!("Error de base de datos: {e}")
    }
}

pub fn obtener_o_crear_categoria(tx: &Connection, nombre: &str) -> Result<i64, String> {
    if let Some(id) = tx
        .query_row(
            "SELECT id FROM categorias WHERE nombre = ?1",
            [nombre],
            |r| r.get(0),
        )
        .optional()
        .map_err(|e| traducir_error_sql(&e))?
    {
        return Ok(id);
    }
    tx.execute("INSERT INTO categorias (nombre) VALUES (?1)", [nombre])
        .map_err(|e| traducir_error_sql(&e))?;
    Ok(tx.last_insert_rowid())
}

pub fn obtener_o_crear_proveedor(tx: &Connection, razon_social: &str) -> Result<i64, String> {
    if let Some(id) = tx
        .query_row(
            "SELECT id FROM proveedores WHERE razon_social = ?1",
            [razon_social],
            |r| r.get(0),
        )
        .optional()
        .map_err(|e| traducir_error_sql(&e))?
    {
        return Ok(id);
    }
    tx.execute(
        "INSERT INTO proveedores (razon_social, activo) VALUES (?1, 1)",
        [razon_social],
    )
    .map_err(|e| traducir_error_sql(&e))?;
    Ok(tx.last_insert_rowid())
}

pub fn obtener_o_crear_cliente_por_documento(
    tx: &Connection,
    documento: &str,
    nombre_si_nuevo: &str,
) -> Result<i64, String> {
    if let Some(id) = tx
        .query_row(
            "SELECT id FROM clientes WHERE documento = ?1",
            [documento],
            |r| r.get(0),
        )
        .optional()
        .map_err(|e| traducir_error_sql(&e))?
    {
        return Ok(id);
    }
    tx.execute(
        "INSERT INTO clientes (documento, nombre) VALUES (?1, ?2)",
        (documento, nombre_si_nuevo),
    )
    .map_err(|e| traducir_error_sql(&e))?;
    Ok(tx.last_insert_rowid())
}

pub fn buscar_producto_por_sku(tx: &Connection, sku: &str) -> Result<Option<i64>, String> {
    tx.query_row("SELECT id FROM productos WHERE sku = ?1", [sku], |r| {
        r.get(0)
    })
    .optional()
    .map_err(|e| traducir_error_sql(&e))
}

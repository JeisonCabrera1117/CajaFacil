use rusqlite::types::Value;
use rusqlite::{params, OptionalExtension};

use crate::db::DbPool;
use crate::error::{AppError, AppResult};
use crate::models::producto::{Producto, ProductoFiltro, ProductoNuevo, ProductoPagina};
use crate::services::audit_service;

pub(crate) const COLUMNAS: &str =
    "id, sku, codigo_barras, nombre, descripcion, categoria_id, unidad_medida,
    precio_costo, precio_venta, impuesto_pct, stock_actual, stock_minimo, stock_maximo,
    ubicacion, proveedor_principal_id, estado, imagen_path";

pub(crate) fn map(row: &rusqlite::Row) -> rusqlite::Result<Producto> {
    Ok(Producto {
        id: row.get(0)?,
        sku: row.get(1)?,
        codigo_barras: row.get(2)?,
        nombre: row.get(3)?,
        descripcion: row.get(4)?,
        categoria_id: row.get(5)?,
        unidad_medida: row.get(6)?,
        precio_costo: row.get(7)?,
        precio_venta: row.get(8)?,
        impuesto_pct: row.get(9)?,
        stock_actual: row.get(10)?,
        stock_minimo: row.get(11)?,
        stock_maximo: row.get(12)?,
        ubicacion: row.get(13)?,
        proveedor_principal_id: row.get(14)?,
        estado: row.get(15)?,
        imagen_path: row.get(16)?,
    })
}

pub fn listar(pool: &DbPool, filtro: ProductoFiltro) -> AppResult<ProductoPagina> {
    let conn = pool.get()?;

    let mut condiciones: Vec<String> = vec!["1=1".to_string()];
    let mut valores: Vec<Value> = Vec::new();

    if let Some(busqueda) = filtro
        .busqueda
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        condiciones.push("(nombre LIKE ? OR sku LIKE ? OR codigo_barras LIKE ?)".into());
        let patron = format!("%{busqueda}%");
        valores.push(Value::Text(patron.clone()));
        valores.push(Value::Text(patron.clone()));
        valores.push(Value::Text(patron));
    }
    if let Some(categoria_id) = filtro.categoria_id {
        condiciones.push("categoria_id = ?".into());
        valores.push(Value::Integer(categoria_id));
    }
    if filtro.solo_stock_bajo.unwrap_or(false) {
        condiciones.push("stock_actual <= stock_minimo".into());
    }
    if filtro.solo_sobre_stock.unwrap_or(false) {
        condiciones.push("(stock_maximo IS NOT NULL AND stock_actual > stock_maximo)".into());
    }

    let where_clause = condiciones.join(" AND ");

    let total: i64 = {
        let sql = format!("SELECT COUNT(*) FROM productos WHERE {where_clause}");
        conn.query_row(&sql, rusqlite::params_from_iter(valores.iter()), |r| {
            r.get(0)
        })?
    };

    let pagina = filtro.pagina.unwrap_or(1).max(1);
    let por_pagina = filtro.por_pagina.unwrap_or(50).clamp(1, 200);
    let offset = (pagina - 1) * por_pagina;

    let sql = format!(
        "SELECT {COLUMNAS} FROM productos WHERE {where_clause} ORDER BY nombre LIMIT ? OFFSET ?"
    );
    let mut parametros = valores;
    parametros.push(Value::Integer(por_pagina));
    parametros.push(Value::Integer(offset));

    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map(rusqlite::params_from_iter(parametros.iter()), map)?;
    let mut items = Vec::new();
    for r in rows {
        items.push(r?);
    }

    Ok(ProductoPagina { items, total })
}

pub fn obtener(pool: &DbPool, id: i64) -> AppResult<Producto> {
    let conn = pool.get()?;
    let sql = format!("SELECT {COLUMNAS} FROM productos WHERE id = ?1");
    conn.query_row(&sql, [id], map)
        .optional()?
        .ok_or_else(|| AppError::NoEncontrado("El producto no existe.".into()))
}

fn validar(datos: &ProductoNuevo) -> AppResult<()> {
    if datos.sku.trim().is_empty() {
        return Err(AppError::Validacion("El SKU es obligatorio.".into()));
    }
    if datos.nombre.trim().is_empty() {
        return Err(AppError::Validacion("El nombre es obligatorio.".into()));
    }
    if datos.precio_costo < 0 || datos.precio_venta < 0 {
        return Err(AppError::Validacion(
            "Los precios no pueden ser negativos.".into(),
        ));
    }
    if datos.stock_minimo < 0 {
        return Err(AppError::Validacion(
            "El stock mínimo no puede ser negativo.".into(),
        ));
    }
    Ok(())
}

fn es_error_restriccion(e: &rusqlite::Error) -> bool {
    matches!(
        e,
        rusqlite::Error::SqliteFailure(err, _) if err.code == rusqlite::ErrorCode::ConstraintViolation
    )
}

fn traducir_error_unicidad(e: rusqlite::Error) -> AppError {
    if let rusqlite::Error::SqliteFailure(_, Some(msg)) = &e {
        if msg.contains("productos.sku") {
            return AppError::Validacion("Ya existe un producto con ese SKU.".into());
        }
        if msg.contains("productos.codigo_barras") {
            return AppError::Validacion("Ya existe un producto con ese código de barras.".into());
        }
    }
    AppError::Db(e)
}

pub fn crear(pool: &DbPool, datos: ProductoNuevo) -> AppResult<Producto> {
    validar(&datos)?;
    let conn = pool.get()?;
    conn.execute(
        "INSERT INTO productos
            (sku, codigo_barras, nombre, descripcion, categoria_id, unidad_medida,
             precio_costo, precio_venta, impuesto_pct, stock_minimo, stock_maximo,
             ubicacion, proveedor_principal_id, estado, imagen_path)
         VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15)",
        params![
            datos.sku,
            datos.codigo_barras,
            datos.nombre,
            datos.descripcion,
            datos.categoria_id,
            datos.unidad_medida,
            datos.precio_costo,
            datos.precio_venta,
            datos.impuesto_pct,
            datos.stock_minimo,
            datos.stock_maximo,
            datos.ubicacion,
            datos.proveedor_principal_id,
            datos.estado,
            datos.imagen_path,
        ],
    )
    .map_err(traducir_error_unicidad)?;
    let id = conn.last_insert_rowid();
    drop(conn);
    let creado = obtener(pool, id)?;
    audit_service::registrar(pool, "producto", Some(id), "crear", &creado)?;
    Ok(creado)
}

pub fn actualizar(pool: &DbPool, id: i64, datos: ProductoNuevo) -> AppResult<Producto> {
    validar(&datos)?;
    let conn = pool.get()?;
    let filas = conn
        .execute(
            "UPDATE productos SET sku=?1, codigo_barras=?2, nombre=?3, descripcion=?4, categoria_id=?5,
                unidad_medida=?6, precio_costo=?7, precio_venta=?8, impuesto_pct=?9, stock_minimo=?10,
                stock_maximo=?11, ubicacion=?12, proveedor_principal_id=?13, estado=?14, imagen_path=?15,
                actualizado_en = datetime('now')
             WHERE id = ?16",
            params![
                datos.sku, datos.codigo_barras, datos.nombre, datos.descripcion, datos.categoria_id,
                datos.unidad_medida, datos.precio_costo, datos.precio_venta, datos.impuesto_pct,
                datos.stock_minimo, datos.stock_maximo, datos.ubicacion, datos.proveedor_principal_id,
                datos.estado, datos.imagen_path, id,
            ],
        )
        .map_err(traducir_error_unicidad)?;
    if filas == 0 {
        return Err(AppError::NoEncontrado("El producto no existe.".into()));
    }
    drop(conn);
    let actualizado = obtener(pool, id)?;
    audit_service::registrar(pool, "producto", Some(id), "modificar", &actualizado)?;
    Ok(actualizado)
}

pub fn eliminar(pool: &DbPool, id: i64) -> AppResult<()> {
    let conn = pool.get()?;
    let filas = conn.execute("DELETE FROM productos WHERE id = ?1", [id]).map_err(|e| {
        if es_error_restriccion(&e) {
            AppError::Validacion(
                "No se puede eliminar: el producto tiene movimientos, compras o ventas asociadas. Puede desactivarlo en su lugar."
                    .into(),
            )
        } else {
            AppError::Db(e)
        }
    })?;
    if filas == 0 {
        return Err(AppError::NoEncontrado("El producto no existe.".into()));
    }
    drop(conn);
    audit_service::registrar(pool, "producto", Some(id), "eliminar", &id)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::producto::ProductoFiltro;
    use tempfile::tempdir;

    fn pool_de_prueba() -> DbPool {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.sqlite3");
        let pool = crate::db::init_pool(&path).unwrap();
        std::mem::forget(dir);
        pool
    }

    fn crear_producto(
        pool: &DbPool,
        sku: &str,
        stock_actual: i64,
        stock_minimo: i64,
        stock_maximo: Option<i64>,
    ) {
        let conn = pool.get().unwrap();
        conn.execute(
            "INSERT INTO productos (sku, nombre, unidad_medida, precio_costo, precio_venta,
                                     stock_actual, stock_minimo, stock_maximo)
             VALUES (?1, 'Producto', 'unidad', 1000, 2000, ?2, ?3, ?4)",
            params![sku, stock_actual, stock_minimo, stock_maximo],
        )
        .unwrap();
    }

    #[test]
    fn filtro_solo_sobre_stock_requiere_maximo_definido_y_superado() {
        let pool = pool_de_prueba();
        crear_producto(&pool, "SKU-P1", 50, 5, Some(20)); // sobre-stock
        crear_producto(&pool, "SKU-P2", 50, 5, None); // sin máximo: no cuenta
        crear_producto(&pool, "SKU-P3", 10, 5, Some(20)); // dentro del máximo

        let pagina = listar(
            &pool,
            ProductoFiltro {
                solo_sobre_stock: Some(true),
                ..Default::default()
            },
        )
        .unwrap();

        assert_eq!(pagina.items.len(), 1);
        assert_eq!(pagina.items[0].sku, "SKU-P1");
    }
}

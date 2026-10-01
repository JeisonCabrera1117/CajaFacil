use rusqlite::{params, Connection};

use crate::db::DbPool;
use crate::error::{AppError, AppResult};
use crate::models::movimiento::MovimientoStock;

const TIPOS_VALIDOS: [&str; 4] = ["entrada", "salida", "ajuste", "devolucion"];

pub struct NuevoMovimiento {
    pub producto_id: i64,
    pub tipo: String,
    pub cantidad: i64,
    pub costo_unitario: Option<i64>,
    pub motivo: Option<String>,
    pub referencia_tipo: Option<String>,
    pub referencia_id: Option<i64>,
}

fn map_movimiento(row: &rusqlite::Row) -> rusqlite::Result<MovimientoStock> {
    Ok(MovimientoStock {
        id: row.get(0)?,
        producto_id: row.get(1)?,
        tipo: row.get(2)?,
        cantidad: row.get(3)?,
        costo_unitario: row.get(4)?,
        motivo: row.get(5)?,
        referencia_tipo: row.get(6)?,
        referencia_id: row.get(7)?,
        fecha: row.get(8)?,
        saldo_resultante: row.get(9)?,
    })
}

/// Aplica un movimiento de stock dentro de una conexión/transacción ya abierta
/// por el llamador (compras, anulaciones, cierre de inventario físico). Para
/// un movimiento aislado use `registrar_movimiento`, que abre su propia
/// transacción.
pub fn aplicar_movimiento_tx(
    conn: &Connection,
    mov: NuevoMovimiento,
) -> AppResult<MovimientoStock> {
    if !TIPOS_VALIDOS.contains(&mov.tipo.as_str()) {
        return Err(AppError::Validacion("Tipo de movimiento inválido.".into()));
    }
    if mov.tipo == "ajuste" {
        if mov.motivo.as_deref().unwrap_or("").trim().is_empty() {
            return Err(AppError::Validacion(
                "El motivo es obligatorio para un ajuste.".into(),
            ));
        }
        if mov.cantidad == 0 {
            return Err(AppError::Validacion(
                "La cantidad del ajuste no puede ser cero.".into(),
            ));
        }
    } else if mov.cantidad <= 0 {
        return Err(AppError::Validacion(
            "La cantidad debe ser mayor que cero.".into(),
        ));
    }

    let delta = match mov.tipo.as_str() {
        "entrada" | "devolucion" => mov.cantidad,
        "salida" => -mov.cantidad,
        "ajuste" => mov.cantidad,
        _ => unreachable!(),
    };

    let (stock_actual, precio_costo, permite_stock_negativo): (i64, i64, i64) = conn
        .query_row(
            "SELECT p.stock_actual, p.precio_costo, e.permite_stock_negativo
             FROM productos p, empresa_config e WHERE p.id = ?1 AND e.id = 1",
            [mov.producto_id],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => {
                AppError::NoEncontrado("El producto no existe.".into())
            }
            other => AppError::Db(other),
        })?;

    let nuevo_stock = stock_actual + delta;
    if nuevo_stock < 0 && permite_stock_negativo == 0 {
        return Err(AppError::Validacion(format!(
            "Stock insuficiente: la operación dejaría {nuevo_stock} unidades y la configuración no permite stock negativo."
        )));
    }

    let nuevo_costo = if mov.tipo == "entrada" && nuevo_stock > 0 {
        match mov.costo_unitario {
            Some(costo_nuevo) => {
                let numerador = stock_actual * precio_costo + mov.cantidad * costo_nuevo;
                (numerador + nuevo_stock / 2) / nuevo_stock
            }
            None => precio_costo,
        }
    } else {
        precio_costo
    };

    conn.execute(
        "UPDATE productos SET stock_actual = ?1, precio_costo = ?2, actualizado_en = datetime('now') WHERE id = ?3",
        params![nuevo_stock, nuevo_costo, mov.producto_id],
    )?;

    conn.execute(
        "INSERT INTO movimientos_stock
            (producto_id, tipo, cantidad, costo_unitario, motivo, referencia_tipo, referencia_id, saldo_resultante)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            mov.producto_id,
            mov.tipo,
            mov.cantidad,
            mov.costo_unitario,
            mov.motivo,
            mov.referencia_tipo,
            mov.referencia_id,
            nuevo_stock,
        ],
    )?;
    let id = conn.last_insert_rowid();

    conn.query_row(
        "SELECT id, producto_id, tipo, cantidad, costo_unitario, motivo, referencia_tipo,
                referencia_id, fecha, saldo_resultante
         FROM movimientos_stock WHERE id = ?1",
        [id],
        map_movimiento,
    )
    .map_err(AppError::from)
}

pub fn registrar_movimiento(pool: &DbPool, mov: NuevoMovimiento) -> AppResult<MovimientoStock> {
    let mut conn = pool.get()?;
    let tx = conn.transaction()?;
    let resultado = aplicar_movimiento_tx(&tx, mov)?;
    tx.commit()?;
    Ok(resultado)
}

pub fn kardex(pool: &DbPool, producto_id: i64) -> AppResult<Vec<MovimientoStock>> {
    let conn = pool.get()?;
    let mut stmt = conn.prepare(
        "SELECT id, producto_id, tipo, cantidad, costo_unitario, motivo, referencia_tipo,
                referencia_id, fecha, saldo_resultante
         FROM movimientos_stock WHERE producto_id = ?1 ORDER BY fecha DESC, id DESC",
    )?;
    let rows = stmt.query_map([producto_id], map_movimiento)?;
    let mut out = Vec::new();
    for r in rows {
        out.push(r?);
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::init_pool;
    use tempfile::tempdir;

    fn pool_de_prueba() -> DbPool {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.sqlite3");
        let pool = init_pool(&path).unwrap();
        std::mem::forget(dir);
        pool
    }

    fn crear_producto(pool: &DbPool, sku: &str, costo: i64, precio: i64) -> i64 {
        let conn = pool.get().unwrap();
        conn.execute(
            "INSERT INTO productos (sku, nombre, unidad_medida, precio_costo, precio_venta)
             VALUES (?1, 'Producto de prueba', 'unidad', ?2, ?3)",
            params![sku, costo, precio],
        )
        .unwrap();
        conn.last_insert_rowid()
    }

    #[test]
    fn entrada_aumenta_stock_y_promedia_costo() {
        let pool = pool_de_prueba();
        let producto_id = crear_producto(&pool, "SKU-1", 1000, 2000);

        registrar_movimiento(
            &pool,
            NuevoMovimiento {
                producto_id,
                tipo: "entrada".into(),
                cantidad: 10,
                costo_unitario: Some(1000),
                motivo: None,
                referencia_tipo: None,
                referencia_id: None,
            },
        )
        .unwrap();

        registrar_movimiento(
            &pool,
            NuevoMovimiento {
                producto_id,
                tipo: "entrada".into(),
                cantidad: 10,
                costo_unitario: Some(2000),
                motivo: None,
                referencia_tipo: None,
                referencia_id: None,
            },
        )
        .unwrap();

        let conn = pool.get().unwrap();
        let (stock, costo): (i64, i64) = conn
            .query_row(
                "SELECT stock_actual, precio_costo FROM productos WHERE id = ?1",
                [producto_id],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .unwrap();
        assert_eq!(stock, 20);
        assert_eq!(costo, 1500); // promedio ponderado (10*1000 + 10*2000) / 20
    }

    #[test]
    fn salida_respeta_stock_no_negativo() {
        let pool = pool_de_prueba();
        let producto_id = crear_producto(&pool, "SKU-2", 1000, 2000);

        let resultado = registrar_movimiento(
            &pool,
            NuevoMovimiento {
                producto_id,
                tipo: "salida".into(),
                cantidad: 5,
                costo_unitario: None,
                motivo: None,
                referencia_tipo: None,
                referencia_id: None,
            },
        );
        assert!(resultado.is_err());
    }

    #[test]
    fn ajuste_sin_motivo_falla() {
        let pool = pool_de_prueba();
        let producto_id = crear_producto(&pool, "SKU-3", 1000, 2000);

        let resultado = registrar_movimiento(
            &pool,
            NuevoMovimiento {
                producto_id,
                tipo: "ajuste".into(),
                cantidad: 5,
                costo_unitario: None,
                motivo: None,
                referencia_tipo: None,
                referencia_id: None,
            },
        );
        assert!(resultado.is_err());
    }

    #[test]
    fn kardex_devuelve_saldo_acumulado() {
        let pool = pool_de_prueba();
        let producto_id = crear_producto(&pool, "SKU-4", 1000, 2000);

        registrar_movimiento(
            &pool,
            NuevoMovimiento {
                producto_id,
                tipo: "entrada".into(),
                cantidad: 10,
                costo_unitario: Some(1000),
                motivo: None,
                referencia_tipo: None,
                referencia_id: None,
            },
        )
        .unwrap();
        registrar_movimiento(
            &pool,
            NuevoMovimiento {
                producto_id,
                tipo: "salida".into(),
                cantidad: 4,
                costo_unitario: None,
                motivo: None,
                referencia_tipo: None,
                referencia_id: None,
            },
        )
        .unwrap();

        let movimientos = kardex(&pool, producto_id).unwrap();
        assert_eq!(movimientos.len(), 2);
        assert_eq!(movimientos[0].saldo_resultante, 6);
        assert_eq!(movimientos[1].saldo_resultante, 10);
    }
}

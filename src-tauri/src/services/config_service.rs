use rusqlite::params;

use crate::db::DbPool;
use crate::error::{AppError, AppResult};
use crate::models::config::{EmpresaConfig, EmpresaConfigActualizar};

pub fn obtener(pool: &DbPool) -> AppResult<EmpresaConfig> {
    let conn = pool.get()?;
    let config = conn.query_row(
        "SELECT nombre, nit, direccion, telefono, logo_path, moneda, formato_fecha,
                prefijo_comprobante, siguiente_numero, leyenda_pie, tema,
                requiere_pin, permite_stock_negativo, arqueo_activo
         FROM empresa_config WHERE id = 1",
        [],
        |row| {
            Ok(EmpresaConfig {
                nombre: row.get(0)?,
                nit: row.get(1)?,
                direccion: row.get(2)?,
                telefono: row.get(3)?,
                logo_path: row.get(4)?,
                moneda: row.get(5)?,
                formato_fecha: row.get(6)?,
                prefijo_comprobante: row.get(7)?,
                siguiente_numero: row.get(8)?,
                leyenda_pie: row.get(9)?,
                tema: row.get(10)?,
                requiere_pin: row.get::<_, i64>(11)? != 0,
                permite_stock_negativo: row.get::<_, i64>(12)? != 0,
                arqueo_activo: row.get::<_, i64>(13)? != 0,
            })
        },
    )?;
    Ok(config)
}

pub fn actualizar(pool: &DbPool, datos: EmpresaConfigActualizar) -> AppResult<EmpresaConfig> {
    if datos.nombre.trim().is_empty() {
        return Err(AppError::Validacion(
            "El nombre de la empresa es obligatorio.".into(),
        ));
    }

    let conn = pool.get()?;
    conn.execute(
        "UPDATE empresa_config SET nombre = ?1, nit = ?2, direccion = ?3, telefono = ?4,
            logo_path = ?5, moneda = ?6, formato_fecha = ?7, prefijo_comprobante = ?8,
            leyenda_pie = ?9, tema = ?10, permite_stock_negativo = ?11, arqueo_activo = ?12,
            actualizado_en = datetime('now')
         WHERE id = 1",
        params![
            datos.nombre,
            datos.nit,
            datos.direccion,
            datos.telefono,
            datos.logo_path,
            datos.moneda,
            datos.formato_fecha,
            datos.prefijo_comprobante,
            datos.leyenda_pie,
            datos.tema,
            datos.permite_stock_negativo as i64,
            datos.arqueo_activo as i64,
        ],
    )?;
    drop(conn);
    obtener(pool)
}

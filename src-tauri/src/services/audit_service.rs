use rusqlite::params;
use rusqlite::types::Value;
use serde::{Deserialize, Serialize};

use crate::db::DbPool;
use crate::error::AppResult;

/// Registra una entrada de auditoría. Nunca falla el flujo llamador si algo
/// sale mal serializando el detalle: en el peor caso queda sin detalle.
pub fn registrar<T: Serialize>(
    pool: &DbPool,
    entidad: &str,
    entidad_id: Option<i64>,
    accion: &str,
    detalle: &T,
) -> AppResult<()> {
    let detalle_json = serde_json::to_string(detalle).unwrap_or_default();
    let conn = pool.get()?;
    conn.execute(
        "INSERT INTO auditoria (entidad, entidad_id, accion, detalle_json) VALUES (?1, ?2, ?3, ?4)",
        params![entidad, entidad_id, accion, detalle_json],
    )?;
    Ok(())
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EntradaAuditoria {
    pub id: i64,
    pub fecha: String,
    pub entidad: String,
    pub entidad_id: Option<i64>,
    pub accion: String,
    pub detalle_json: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct FiltroAuditoria {
    pub entidad: Option<String>,
    pub desde: Option<String>,
    pub hasta: Option<String>,
    pub pagina: Option<i64>,
    pub por_pagina: Option<i64>,
}

fn map(row: &rusqlite::Row) -> rusqlite::Result<EntradaAuditoria> {
    Ok(EntradaAuditoria {
        id: row.get(0)?,
        fecha: row.get(1)?,
        entidad: row.get(2)?,
        entidad_id: row.get(3)?,
        accion: row.get(4)?,
        detalle_json: row.get(5)?,
    })
}

pub fn listar(pool: &DbPool, filtro: FiltroAuditoria) -> AppResult<Vec<EntradaAuditoria>> {
    let conn = pool.get()?;

    let mut condiciones = Vec::new();
    let mut valores: Vec<Value> = Vec::new();

    if let Some(entidad) = filtro.entidad.filter(|e| !e.is_empty()) {
        condiciones.push("entidad = ?");
        valores.push(Value::Text(entidad));
    }
    // `fecha` se guarda en UTC; comparar contra el día calendario local que
    // el usuario eligió requiere 'localtime' (ver nota en dashboard_service.rs).
    if let Some(desde) = filtro.desde.filter(|d| !d.is_empty()) {
        condiciones.push("date(fecha,'localtime') >= ?");
        valores.push(Value::Text(desde));
    }
    if let Some(hasta) = filtro.hasta.filter(|d| !d.is_empty()) {
        condiciones.push("date(fecha,'localtime') <= ?");
        valores.push(Value::Text(hasta));
    }

    let where_sql = if condiciones.is_empty() {
        String::new()
    } else {
        format!("WHERE {}", condiciones.join(" AND "))
    };

    let por_pagina = filtro.por_pagina.unwrap_or(100).clamp(1, 500);
    let pagina = filtro.pagina.unwrap_or(1).max(1);
    let offset = (pagina - 1) * por_pagina;
    valores.push(Value::Integer(por_pagina));
    valores.push(Value::Integer(offset));

    let sql = format!(
        "SELECT id, fecha, entidad, entidad_id, accion, detalle_json FROM auditoria \
         {where_sql} ORDER BY fecha DESC, id DESC LIMIT ? OFFSET ?"
    );

    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map(rusqlite::params_from_iter(valores.iter()), map)?;
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

    #[test]
    fn registra_y_lista_sin_filtro() {
        let pool = pool_de_prueba();
        registrar(&pool, "producto", Some(1), "crear", &"detalle").unwrap();
        registrar(&pool, "categoria", Some(2), "crear", &"detalle").unwrap();

        let todas = listar(&pool, FiltroAuditoria::default()).unwrap();
        assert_eq!(todas.len(), 2);
    }

    #[test]
    fn filtra_por_entidad() {
        let pool = pool_de_prueba();
        registrar(&pool, "producto", Some(1), "crear", &"a").unwrap();
        registrar(&pool, "categoria", Some(2), "crear", &"b").unwrap();

        let filtro = FiltroAuditoria {
            entidad: Some("producto".into()),
            ..Default::default()
        };
        let filtradas = listar(&pool, filtro).unwrap();
        assert_eq!(filtradas.len(), 1);
        assert_eq!(filtradas[0].entidad, "producto");
    }

    #[test]
    fn pagina_correctamente() {
        let pool = pool_de_prueba();
        for i in 0..5 {
            registrar(&pool, "producto", Some(i), "crear", &i).unwrap();
        }
        let filtro = FiltroAuditoria {
            pagina: Some(2),
            por_pagina: Some(2),
            ..Default::default()
        };
        let pagina = listar(&pool, filtro).unwrap();
        assert_eq!(pagina.len(), 2);
    }
}

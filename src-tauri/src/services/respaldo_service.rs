use std::path::Path;

use chrono::Local;
use rusqlite::OptionalExtension;
use serde::Serialize;

use crate::db::DbPool;
use crate::error::{AppError, AppResult};

/// Cuántos respaldos se conservan como máximo (los más viejos se eliminan al crear uno nuevo).
const LIMITE_RESPALDOS: i64 = 20;

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Respaldo {
    pub id: i64,
    pub fecha: String,
    pub ruta_archivo: String,
    pub tipo: String,
    pub tamano_bytes: i64,
}

fn map(row: &rusqlite::Row) -> rusqlite::Result<Respaldo> {
    Ok(Respaldo {
        id: row.get(0)?,
        fecha: row.get(1)?,
        ruta_archivo: row.get(2)?,
        tipo: row.get(3)?,
        tamano_bytes: row.get(4)?,
    })
}

pub fn listar(pool: &DbPool) -> AppResult<Vec<Respaldo>> {
    let conn = pool.get()?;
    let mut stmt = conn.prepare(
        "SELECT id, fecha, ruta_archivo, tipo, tamano_bytes FROM respaldos ORDER BY fecha DESC",
    )?;
    let rows = stmt.query_map([], map)?;
    let mut out = Vec::new();
    for r in rows {
        out.push(r?);
    }
    Ok(out)
}

/// Crea un respaldo consistente de la base de datos usando `VACUUM INTO`, que
/// no requiere bloquear escrituras concurrentes ni cerrar la conexión activa.
pub fn crear(pool: &DbPool, respaldos_dir: &Path, tipo: &str) -> AppResult<Respaldo> {
    std::fs::create_dir_all(respaldos_dir)
        .map_err(|e| AppError::Interno(format!("No se pudo crear la carpeta de respaldos: {e}")))?;

    let nombre = format!(
        "respaldo_{}_{}.sqlite3",
        tipo,
        Local::now().format("%Y%m%d_%H%M%S")
    );
    let destino = respaldos_dir.join(&nombre);
    let destino_str = destino.to_string_lossy().to_string();

    let conn = pool.get()?;
    conn.execute("VACUUM INTO ?1", [&destino_str])?;
    drop(conn);

    let tamano_bytes = std::fs::metadata(&destino)
        .map(|m| m.len() as i64)
        .unwrap_or(0);

    let conn = pool.get()?;
    conn.execute(
        "INSERT INTO respaldos (ruta_archivo, tipo, tamano_bytes) VALUES (?1, ?2, ?3)",
        rusqlite::params![destino_str, tipo, tamano_bytes],
    )?;
    let id = conn.last_insert_rowid();
    drop(conn);

    podar(pool, LIMITE_RESPALDOS)?;

    let conn = pool.get()?;
    conn.query_row(
        "SELECT id, fecha, ruta_archivo, tipo, tamano_bytes FROM respaldos WHERE id = ?1",
        [id],
        map,
    )
    .map_err(AppError::from)
}

/// Elimina los respaldos más antiguos que excedan el límite dado (fila y archivo).
pub fn podar(pool: &DbPool, limite: i64) -> AppResult<()> {
    let conn = pool.get()?;
    let mut stmt = conn
        .prepare("SELECT id, ruta_archivo FROM respaldos ORDER BY fecha DESC LIMIT -1 OFFSET ?1")?;
    let sobrantes: Vec<(i64, String)> = stmt
        .query_map([limite], |r| Ok((r.get(0)?, r.get(1)?)))?
        .collect::<Result<_, _>>()?;
    drop(stmt);
    drop(conn);

    for (id, ruta) in sobrantes {
        std::fs::remove_file(&ruta).ok();
        let conn = pool.get()?;
        conn.execute("DELETE FROM respaldos WHERE id = ?1", [id])?;
    }
    Ok(())
}

pub fn eliminar(pool: &DbPool, id: i64) -> AppResult<()> {
    let conn = pool.get()?;
    let ruta: Option<String> = conn
        .query_row(
            "SELECT ruta_archivo FROM respaldos WHERE id = ?1",
            [id],
            |r| r.get(0),
        )
        .optional()?;
    let Some(ruta) = ruta else {
        return Err(AppError::NoEncontrado("El respaldo no existe.".into()));
    };
    conn.execute("DELETE FROM respaldos WHERE id = ?1", [id])?;
    drop(conn);
    std::fs::remove_file(&ruta).ok();
    Ok(())
}

/// No restaura de inmediato (la conexión activa de la app sigue con el archivo
/// de base de datos abierto). En vez de eso, deja un archivo "marcador" con la
/// ruta del respaldo elegido; al reiniciar, `lib.rs` lo detecta antes de abrir
/// el pool, copia el respaldo sobre la base de datos real y borra el marcador.
pub fn solicitar_restauracion(pool: &DbPool, app_data_dir: &Path, id: i64) -> AppResult<()> {
    let conn = pool.get()?;
    let ruta: Option<String> = conn
        .query_row(
            "SELECT ruta_archivo FROM respaldos WHERE id = ?1",
            [id],
            |r| r.get(0),
        )
        .optional()?;
    drop(conn);
    let Some(ruta) = ruta else {
        return Err(AppError::NoEncontrado("El respaldo no existe.".into()));
    };
    if !Path::new(&ruta).is_file() {
        return Err(AppError::Validacion(
            "El archivo de ese respaldo ya no existe en el disco.".into(),
        ));
    }
    let marcador = app_data_dir.join("restaurar.marcador");
    std::fs::write(&marcador, ruta).map_err(|e| {
        AppError::Interno(format!("No se pudo dejar el marcador de restauración: {e}"))
    })?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::init_pool;
    use tempfile::tempdir;

    fn entorno_de_prueba() -> (DbPool, std::path::PathBuf) {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.sqlite3");
        let pool = init_pool(&db_path).unwrap();
        let respaldos_dir = dir.path().join("respaldos");
        std::mem::forget(dir);
        (pool, respaldos_dir)
    }

    #[test]
    fn crea_respaldo_y_queda_listado() {
        let (pool, dir) = entorno_de_prueba();
        let r = crear(&pool, &dir, "manual").unwrap();
        assert!(Path::new(&r.ruta_archivo).is_file());
        assert!(r.tamano_bytes > 0);

        let lista = listar(&pool).unwrap();
        assert_eq!(lista.len(), 1);
        assert_eq!(lista[0].id, r.id);
    }

    #[test]
    fn eliminar_respaldo_borra_fila_y_archivo() {
        let (pool, dir) = entorno_de_prueba();
        let r = crear(&pool, &dir, "manual").unwrap();
        eliminar(&pool, r.id).unwrap();
        assert!(!Path::new(&r.ruta_archivo).is_file());
        assert!(listar(&pool).unwrap().is_empty());
    }

    #[test]
    fn eliminar_respaldo_inexistente_falla() {
        let (pool, _dir) = entorno_de_prueba();
        assert!(eliminar(&pool, 999).is_err());
    }

    #[test]
    fn podar_conserva_solo_los_mas_recientes() {
        let (pool, dir) = entorno_de_prueba();
        for _ in 0..5 {
            crear(&pool, &dir, "manual").unwrap();
            std::thread::sleep(std::time::Duration::from_millis(1100));
        }
        podar(&pool, 2).unwrap();
        assert_eq!(listar(&pool).unwrap().len(), 2);
    }

    #[test]
    fn solicitar_restauracion_deja_marcador_con_la_ruta() {
        let (pool, dir) = entorno_de_prueba();
        let r = crear(&pool, &dir, "manual").unwrap();
        let app_data_dir = tempdir().unwrap();
        solicitar_restauracion(&pool, app_data_dir.path(), r.id).unwrap();
        let contenido =
            std::fs::read_to_string(app_data_dir.path().join("restaurar.marcador")).unwrap();
        assert_eq!(contenido, r.ruta_archivo);
    }

    #[test]
    fn solicitar_restauracion_de_id_inexistente_falla() {
        let (pool, _dir) = entorno_de_prueba();
        let app_data_dir = tempdir().unwrap();
        assert!(solicitar_restauracion(&pool, app_data_dir.path(), 999).is_err());
    }
}

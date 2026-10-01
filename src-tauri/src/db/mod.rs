use std::path::Path;
use std::sync::LazyLock;

use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite_migration::{Migrations, M};

use crate::error::AppResult;

pub type DbPool = Pool<SqliteConnectionManager>;

static MIGRATIONS: LazyLock<Migrations<'static>> = LazyLock::new(|| {
    Migrations::new(vec![M::up(include_str!(
        "../../migrations/0001_initial.sql"
    ))])
});

pub fn init_pool(db_path: &Path) -> AppResult<DbPool> {
    let manager = SqliteConnectionManager::file(db_path).with_init(|conn| {
        conn.execute_batch("PRAGMA foreign_keys = ON; PRAGMA journal_mode = WAL;")
    });
    // SQLite serializa las escrituras de todos modos; con una sola conexión evitamos
    // contención de "database is locked" al abrir varias conexiones simultáneas en frío.
    let pool = Pool::builder().max_size(1).build(manager)?;

    let mut conn = pool.get()?;
    MIGRATIONS.to_latest(&mut conn)?;

    Ok(pool)
}

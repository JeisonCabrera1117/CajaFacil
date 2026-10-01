use std::path::PathBuf;

use crate::db::DbPool;

pub struct AppState {
    pub pool: DbPool,
    pub comprobantes_dir: PathBuf,
    pub importaciones_dir: PathBuf,
    pub respaldos_dir: PathBuf,
}

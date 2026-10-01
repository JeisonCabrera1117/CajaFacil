use serde::{Serialize, Serializer};

/// Error de aplicación. El mensaje técnico se registra en el log; al frontend
/// solo cruza un mensaje amigable en español (ver `mensaje_usuario`).
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Error de base de datos: {0}")]
    Db(#[from] rusqlite::Error),
    #[error("Error de pool de conexiones: {0}")]
    Pool(#[from] r2d2::Error),
    #[error("Error de migraciones: {0}")]
    Migration(#[from] rusqlite_migration::Error),
    #[error("{0}")]
    Validacion(String),
    // Se usará a partir de la fase 2, cuando existan comandos de búsqueda por id.
    #[allow(dead_code)]
    #[error("{0}")]
    NoEncontrado(String),
    #[error("Error interno: {0}")]
    Interno(String),
}

impl AppError {
    fn mensaje_usuario(&self) -> String {
        match self {
            AppError::Db(_) | AppError::Pool(_) | AppError::Migration(_) => {
                "Ocurrió un error de base de datos. Revise el archivo de registro para más detalles.".to_string()
            }
            AppError::Interno(_) => {
                "Ocurrió un error interno. Revise el archivo de registro para más detalles.".to_string()
            }
            AppError::Validacion(msg) | AppError::NoEncontrado(msg) => msg.clone(),
        }
    }
}

impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        tracing::error!(error = %self, "comando falló");
        serializer.serialize_str(&self.mensaje_usuario())
    }
}

pub type AppResult<T> = Result<T, AppError>;

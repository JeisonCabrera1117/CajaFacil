use argon2::password_hash::{rand_core::OsRng, SaltString};
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};

use crate::db::DbPool;
use crate::error::{AppError, AppResult};

pub fn pin_requerido(pool: &DbPool) -> AppResult<bool> {
    let conn = pool.get()?;
    let requiere: i64 = conn.query_row(
        "SELECT requiere_pin FROM empresa_config WHERE id = 1",
        [],
        |r| r.get(0),
    )?;
    Ok(requiere != 0)
}

pub fn establecer_pin(pool: &DbPool, pin: &str) -> AppResult<()> {
    validar_formato_pin(pin)?;

    let salt = SaltString::generate(&mut OsRng);
    let hash = Argon2::default()
        .hash_password(pin.as_bytes(), &salt)
        .map_err(|e| AppError::Interno(format!("No se pudo generar el hash del PIN: {e}")))?
        .to_string();

    let conn = pool.get()?;
    conn.execute(
        "UPDATE empresa_config SET pin_hash = ?1, requiere_pin = 1, actualizado_en = datetime('now') WHERE id = 1",
        [hash],
    )?;
    Ok(())
}

pub fn deshabilitar_pin(pool: &DbPool, pin_actual: &str) -> AppResult<()> {
    if !verificar_pin(pool, pin_actual)? {
        return Err(AppError::Validacion(
            "El PIN ingresado no es correcto.".into(),
        ));
    }
    let conn = pool.get()?;
    conn.execute(
        "UPDATE empresa_config SET pin_hash = NULL, requiere_pin = 0, actualizado_en = datetime('now') WHERE id = 1",
        [],
    )?;
    Ok(())
}

pub fn verificar_pin(pool: &DbPool, pin: &str) -> AppResult<bool> {
    let conn = pool.get()?;
    let hash_guardado: Option<String> = conn.query_row(
        "SELECT pin_hash FROM empresa_config WHERE id = 1",
        [],
        |r| r.get(0),
    )?;

    let Some(hash_guardado) = hash_guardado else {
        return Err(AppError::Validacion("No hay un PIN configurado.".into()));
    };

    let parsed = PasswordHash::new(&hash_guardado)
        .map_err(|e| AppError::Interno(format!("Hash de PIN inválido: {e}")))?;
    Ok(Argon2::default()
        .verify_password(pin.as_bytes(), &parsed)
        .is_ok())
}

fn validar_formato_pin(pin: &str) -> AppResult<()> {
    let longitud_valida = (4..=8).contains(&pin.len());
    if !longitud_valida || !pin.chars().all(|c| c.is_ascii_digit()) {
        return Err(AppError::Validacion(
            "El PIN debe tener entre 4 y 8 dígitos numéricos.".into(),
        ));
    }
    Ok(())
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
    fn establecer_y_verificar_pin() {
        let pool = pool_de_prueba();
        assert!(!pin_requerido(&pool).unwrap());

        establecer_pin(&pool, "1234").unwrap();
        assert!(pin_requerido(&pool).unwrap());
        assert!(verificar_pin(&pool, "1234").unwrap());
        assert!(!verificar_pin(&pool, "0000").unwrap());
    }

    #[test]
    fn rechaza_pin_con_formato_invalido() {
        let pool = pool_de_prueba();
        assert!(establecer_pin(&pool, "abc").is_err());
        assert!(establecer_pin(&pool, "123").is_err());
    }

    #[test]
    fn deshabilitar_pin_requiere_pin_correcto() {
        let pool = pool_de_prueba();
        establecer_pin(&pool, "1234").unwrap();
        assert!(deshabilitar_pin(&pool, "0000").is_err());
        deshabilitar_pin(&pool, "1234").unwrap();
        assert!(!pin_requerido(&pool).unwrap());
    }
}

use crate::db::DbPool;
use crate::error::AppResult;
use crate::models::venta::VentaConDetalle;
use crate::services::{config_service, venta_service};

pub struct DatosComprobante {
    pub empresa_nombre: String,
    pub empresa_nit: String,
    pub empresa_direccion: String,
    pub empresa_telefono: String,
    pub logo_path: Option<String>,
    pub moneda: String,
    pub formato_fecha: String,
    pub leyenda_pie: String,
    pub venta: VentaConDetalle,
}

pub fn construir(pool: &DbPool, venta_id: i64) -> AppResult<DatosComprobante> {
    let config = config_service::obtener(pool)?;
    let venta = venta_service::obtener(pool, venta_id)?;
    Ok(DatosComprobante {
        empresa_nombre: config.nombre,
        empresa_nit: config.nit,
        empresa_direccion: config.direccion,
        empresa_telefono: config.telefono,
        logo_path: config.logo_path,
        moneda: config.moneda,
        formato_fecha: config.formato_fecha,
        leyenda_pie: config.leyenda_pie,
        venta,
    })
}

/// Agrupa los dígitos de a tres desde la derecha con puntos, como en Colombia
/// (12345 -> "12.345").
fn agrupar_miles(n: i64) -> String {
    let texto = n.to_string();
    let bytes = texto.as_bytes();
    let mut salida = String::new();
    for (i, b) in bytes.iter().enumerate() {
        if i > 0 && (bytes.len() - i).is_multiple_of(3) {
            salida.push('.');
        }
        salida.push(*b as char);
    }
    salida
}

/// Espejo de `formatearMoneda` en el frontend (`src/lib/format.ts`): COP sin
/// decimales, cualquier otra moneda con dos decimales separados por coma.
pub fn formatear_moneda(centavos: i64, moneda: &str) -> String {
    let negativo = centavos < 0;
    let absoluto = centavos.unsigned_abs() as i64;
    let entero = absoluto / 100;
    let signo = if negativo { "-" } else { "" };
    if moneda == "COP" {
        format!("{signo}$ {}", agrupar_miles(entero))
    } else {
        let decimales = absoluto % 100;
        format!("{signo}$ {},{decimales:02}", agrupar_miles(entero))
    }
}

/// Espejo de `formatearFechaHora` en el frontend. `fecha_sqlite` viene como
/// "YYYY-MM-DD HH:MM:SS" **en UTC** (default de SQLite `datetime('now')`,
/// ver nota en `services/dashboard_service.rs`) — hay que convertirla a la
/// hora local antes de mostrarla, si no el comprobante impreso sale con la
/// hora de Greenwich (y a veces hasta con la fecha de mañana).
pub fn formatear_fecha(fecha_sqlite: &str, formato: &str) -> String {
    use chrono::{Local, TimeZone, Utc};

    let patron = match formato {
        "MM/DD/YYYY" => "%m/%d/%Y %H:%M",
        "YYYY-MM-DD" => "%Y-%m-%d %H:%M",
        _ => "%d/%m/%Y %H:%M",
    };
    chrono::NaiveDateTime::parse_from_str(fecha_sqlite, "%Y-%m-%d %H:%M:%S")
        .map(|naive_utc| {
            Utc.from_utc_datetime(&naive_utc)
                .with_timezone(&Local)
                .format(patron)
                .to_string()
        })
        .unwrap_or_else(|_| fecha_sqlite.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formatea_cop_sin_decimales_y_con_miles() {
        assert_eq!(formatear_moneda(123_456_700, "COP"), "$ 1.234.567");
        assert_eq!(formatear_moneda(50_000, "COP"), "$ 500");
    }

    #[test]
    fn formatea_otras_monedas_con_decimales() {
        assert_eq!(formatear_moneda(150_050, "USD"), "$ 1.500,50");
    }

    #[test]
    fn formatea_negativos() {
        assert_eq!(formatear_moneda(-50_000, "COP"), "-$ 500");
    }

    #[test]
    fn formatea_fecha_segun_configuracion() {
        // El valor esperado se calcula haciendo la misma conversión UTC->local
        // que la función (en vez de codificar una hora local fija), para que
        // el test pase sin importar en qué huso horario corra la máquina.
        use chrono::{Local, TimeZone, Utc};
        let esperado_local = Utc
            .with_ymd_and_hms(2026, 3, 5, 14, 32, 10)
            .unwrap()
            .with_timezone(&Local);

        assert_eq!(
            formatear_fecha("2026-03-05 14:32:10", "DD/MM/YYYY"),
            esperado_local.format("%d/%m/%Y %H:%M").to_string()
        );
        assert_eq!(
            formatear_fecha("2026-03-05 14:32:10", "YYYY-MM-DD"),
            esperado_local.format("%Y-%m-%d %H:%M").to_string()
        );
    }
}

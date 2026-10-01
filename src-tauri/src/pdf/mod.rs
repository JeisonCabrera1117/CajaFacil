pub mod datos;
pub mod elementos;
pub mod fuente;
pub mod render_imagen;
pub mod render_pdf;

use std::path::{Path, PathBuf};

use chrono::{Datelike, Local, TimeZone, Utc};

use crate::error::{AppError, AppResult};

/// Construye la ruta destino del comprobante, organizada en carpetas por año
/// y mes según la fecha de la venta (no la fecha de generación, para que
/// regenerar un comprobante viejo lo guarde junto a los de su propio mes).
/// `fecha_venta` llega en UTC (ver nota en `services/dashboard_service.rs`):
/// se convierte a la fecha local antes de partirla en año/mes, si no una
/// venta de fin de mes hecha en la noche queda archivada en el mes siguiente.
pub fn ruta_para(
    comprobantes_dir: &Path,
    numero_comprobante: &str,
    tipo: &str,
    fecha_venta: &str,
) -> AppResult<PathBuf> {
    let fecha_local = chrono::NaiveDateTime::parse_from_str(fecha_venta, "%Y-%m-%d %H:%M:%S")
        .map(|naive_utc| Utc.from_utc_datetime(&naive_utc).with_timezone(&Local))
        .map_err(|_| AppError::Interno("Fecha de venta con formato inesperado.".into()))?;
    let anio = fecha_local.year().to_string();
    let mes = format!("{:02}", fecha_local.month());

    let extension = if tipo == "imagen" { "png" } else { "pdf" };
    let nombre_archivo = sanear_nombre(&format!("{numero_comprobante}-{tipo}.{extension}"));
    Ok(comprobantes_dir.join(anio).join(mes).join(nombre_archivo))
}

fn sanear_nombre(nombre: &str) -> String {
    nombre
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '.' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

/// El logo se pasa como ruta (no como imagen ya cargada) porque `printpdf` y
/// el renderer de imagen usan versiones distintas del crate `image` por
/// debajo; cada renderer la carga con la versión que necesita y, si falla
/// (ruta vacía, archivo inexistente, formato inválido), lo omite en silencio
/// — no tiene sentido que falle toda la venta por un logo mal configurado.
pub fn generar_bytes(
    elementos: &[elementos::Elemento],
    logo_path: Option<&str>,
    tipo: &str,
) -> AppResult<Vec<u8>> {
    let logo_path = logo_path.filter(|p| !p.trim().is_empty());
    match tipo {
        "carta" => render_pdf::renderizar_carta(elementos, logo_path),
        "termico80" => render_pdf::renderizar_termico(elementos, logo_path),
        "imagen" => render_imagen::renderizar(elementos, logo_path),
        otro => Err(AppError::Validacion(format!(
            "Tipo de comprobante inválido: {otro}"
        ))),
    }
}

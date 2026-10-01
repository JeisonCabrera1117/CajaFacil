mod tabla_pdf;

use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use rust_xlsxwriter::{Format, Workbook};

use crate::error::{AppError, AppResult};
use crate::models::importacion::PlantillaGenerada;

fn sanear_nombre(titulo: &str) -> String {
    let limpio: String = titulo
        .chars()
        .map(|c| {
            if c.is_alphanumeric() {
                c.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect();
    limpio
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

fn csv_escapar(valor: &str) -> String {
    if valor.contains(',') || valor.contains('"') || valor.contains('\n') {
        format!("\"{}\"", valor.replace('"', "\"\""))
    } else {
        valor.to_string()
    }
}

fn generar_csv(columnas: &[String], filas: &[Vec<String>]) -> Vec<u8> {
    let mut out = String::new();
    out.push_str(
        &columnas
            .iter()
            .map(|c| csv_escapar(c))
            .collect::<Vec<_>>()
            .join(","),
    );
    out.push_str("\r\n");
    for fila in filas {
        out.push_str(
            &fila
                .iter()
                .map(|c| csv_escapar(c))
                .collect::<Vec<_>>()
                .join(","),
        );
        out.push_str("\r\n");
    }
    out.into_bytes()
}

/// Excel limita el nombre de hoja a 31 caracteres y prohíbe ciertos símbolos.
fn nombre_hoja_valido(titulo: &str) -> String {
    let limpio: String = titulo
        .chars()
        .filter(|c| !"[]:*?/\\".contains(*c))
        .collect();
    let recortado: String = limpio.chars().take(31).collect();
    if recortado.trim().is_empty() {
        "Reporte".to_string()
    } else {
        recortado
    }
}

fn generar_xlsx(titulo: &str, columnas: &[String], filas: &[Vec<String>]) -> AppResult<Vec<u8>> {
    let mut libro = Workbook::new();
    let hoja = libro.add_worksheet();
    let _ = hoja.set_name(nombre_hoja_valido(titulo));

    let negrita = Format::new().set_bold();
    for (col, encabezado) in columnas.iter().enumerate() {
        hoja.write_string_with_format(0, col as u16, encabezado, &negrita)
            .map_err(|e| AppError::Interno(format!("Error generando el XLSX: {e}")))?;
    }
    for (fila_idx, fila) in filas.iter().enumerate() {
        for (col, valor) in fila.iter().enumerate() {
            hoja.write_string(fila_idx as u32 + 1, col as u16, valor)
                .map_err(|e| AppError::Interno(format!("Error generando el XLSX: {e}")))?;
        }
    }

    libro
        .save_to_buffer()
        .map_err(|e| AppError::Interno(format!("Error generando el XLSX: {e}")))
}

pub fn generar(
    titulo: &str,
    columnas: &[String],
    filas: &[Vec<String>],
    formato: &str,
) -> AppResult<PlantillaGenerada> {
    let (bytes, mime, extension) = match formato {
        "csv" => (generar_csv(columnas, filas), "text/csv".to_string(), "csv"),
        "xlsx" => (
            generar_xlsx(titulo, columnas, filas)?,
            "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet".to_string(),
            "xlsx",
        ),
        "pdf" => (
            tabla_pdf::generar(titulo, columnas, filas)?,
            "application/pdf".to_string(),
            "pdf",
        ),
        otro => {
            return Err(AppError::Validacion(format!(
                "Formato de exportación inválido: {otro}"
            )))
        }
    };

    Ok(PlantillaGenerada {
        nombre_archivo: format!("{}.{extension}", sanear_nombre(titulo)),
        contenido_base64: STANDARD.encode(bytes),
        mime,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn datos() -> (Vec<String>, Vec<Vec<String>>) {
        (
            vec!["Producto".into(), "Cantidad".into()],
            vec![
                vec!["Coca-Cola, 350ml".into(), "10".into()],
                vec!["Agua \"con gas\"".into(), "5".into()],
            ],
        )
    }

    #[test]
    fn csv_escapa_comas_y_comillas() {
        let (cols, filas) = datos();
        let csv = String::from_utf8(generar_csv(&cols, &filas)).unwrap();
        assert!(csv.contains("\"Coca-Cola, 350ml\""));
        assert!(csv.contains("\"Agua \"\"con gas\"\"\""));
    }

    #[test]
    fn genera_los_tres_formatos_sin_fallar() {
        let (cols, filas) = datos();
        for formato in ["csv", "xlsx", "pdf"] {
            let resultado = generar("Reporte de Ventas", &cols, &filas, formato).unwrap();
            assert!(!resultado.contenido_base64.is_empty(), "{formato} vacío");
            assert_eq!(
                resultado.nombre_archivo,
                format!("reporte-de-ventas.{formato}")
            );
        }
    }

    #[test]
    fn rechaza_formato_invalido() {
        let (cols, filas) = datos();
        assert!(generar("Reporte", &cols, &filas, "docx").is_err());
    }
}

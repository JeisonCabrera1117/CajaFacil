use rust_xlsxwriter::Workbook;

use crate::error::{AppError, AppResult};
use crate::importacion::campos_de;
use crate::models::importacion::PlantillaGenerada;

/// Una fila de ejemplo por entidad, en el mismo orden que `campos_de` — ayuda
/// a que quien complete la plantilla entienda el formato esperado (fechas,
/// números, etc.) sin tener que adivinar.
fn fila_ejemplo(entidad: &str) -> Vec<&'static str> {
    match entidad {
        "productos" => vec![
            "PROD-001",
            "7701234567890",
            "Producto de ejemplo",
            "Descripción opcional",
            "General",
            "unidad",
            "10000",
            "15000",
            "19",
            "5",
            "50",
            "Bodega A",
            "",
            "activo",
        ],
        "categorias" => vec!["Bebidas", ""],
        "proveedores" => {
            vec![
                "900123456-7",
                "Proveedor S.A.S.",
                "Juan Pérez",
                "3001234567",
                "contacto@proveedor.com",
                "Calle 1 # 2-3",
                "30 días",
                "si",
            ]
        }
        "clientes" => vec![
            "1234567890",
            "Cliente de ejemplo",
            "3001234567",
            "cliente@correo.com",
            "Calle 1 # 2-3",
        ],
        "stock_inicial" => vec!["PROD-001", "100", "10000"],
        "ventas_historicas" => {
            vec![
                "2026-01-15",
                "",
                "1234567890",
                "PROD-001",
                "2",
                "15000",
                "0",
                "19",
                "efectivo",
                "35000",
            ]
        }
        _ => Vec::new(),
    }
}

pub fn generar_csv(entidad: &str) -> AppResult<Vec<u8>> {
    let campos = campos_de(entidad)?;
    let encabezados: Vec<String> = campos.iter().map(|c| c.id.clone()).collect();
    let ejemplo = fila_ejemplo(entidad);

    let mut contenido = String::new();
    contenido.push_str(&encabezados.join(","));
    contenido.push_str("\r\n");
    contenido.push_str(&ejemplo.join(","));
    contenido.push_str("\r\n");

    Ok(contenido.into_bytes())
}

pub fn generar_xlsx(entidad: &str) -> AppResult<Vec<u8>> {
    let campos = campos_de(entidad)?;
    let ejemplo = fila_ejemplo(entidad);

    let mut libro = Workbook::new();
    let hoja = libro.add_worksheet();

    for (col, campo) in campos.iter().enumerate() {
        hoja.write_string(0, col as u16, &campo.id)
            .map_err(|e| AppError::Interno(format!("Error generando la plantilla: {e}")))?;
    }
    for (col, valor) in ejemplo.iter().enumerate() {
        hoja.write_string(1, col as u16, *valor)
            .map_err(|e| AppError::Interno(format!("Error generando la plantilla: {e}")))?;
    }

    libro
        .save_to_buffer()
        .map_err(|e| AppError::Interno(format!("Error generando la plantilla: {e}")))
}

pub fn generar(entidad: &str, formato: &str) -> AppResult<PlantillaGenerada> {
    use base64::{engine::general_purpose::STANDARD, Engine};

    let (bytes, mime, extension) = match formato {
        "csv" => (generar_csv(entidad)?, "text/csv", "csv"),
        "xlsx" => (
            generar_xlsx(entidad)?,
            "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
            "xlsx",
        ),
        otro => {
            return Err(AppError::Validacion(format!(
                "Formato de plantilla inválido: {otro}"
            )))
        }
    };

    Ok(PlantillaGenerada {
        nombre_archivo: format!("plantilla-{entidad}.{extension}"),
        contenido_base64: STANDARD.encode(bytes),
        mime: mime.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::importacion;

    #[test]
    fn genera_csv_y_xlsx_para_todas_las_entidades() {
        for entidad in importacion::entidades_importables() {
            let csv = generar_csv(&entidad.id).unwrap();
            assert!(!csv.is_empty(), "csv vacío para {}", entidad.id);
            let texto = String::from_utf8(csv).unwrap();
            assert_eq!(
                texto.lines().count(),
                2,
                "debe tener encabezado + una fila de ejemplo ({})",
                entidad.id
            );

            let xlsx = generar_xlsx(&entidad.id).unwrap();
            assert!(!xlsx.is_empty(), "xlsx vacío para {}", entidad.id);
            assert_eq!(
                &xlsx[0..2],
                b"PK",
                "un .xlsx es un zip, debe empezar con la firma PK ({})",
                entidad.id
            );
        }
    }

    #[test]
    fn rechaza_entidad_desconocida() {
        assert!(generar_csv("no_existe").is_err());
    }
}

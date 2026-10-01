use std::path::Path;

use calamine::{open_workbook_auto, Data, Reader};

use crate::error::{AppError, AppResult};

pub struct ArchivoLeido {
    pub encabezados: Vec<String>,
    pub filas: Vec<Vec<String>>,
}

pub fn es_xlsx(ruta: &str) -> bool {
    let ext = Path::new(ruta)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();
    ext == "xlsx" || ext == "xls" || ext == "xlsm"
}

pub fn listar_hojas(ruta: &str) -> AppResult<Vec<String>> {
    let libro = open_workbook_auto(ruta)
        .map_err(|e| AppError::Validacion(format!("No se pudo abrir el archivo: {e}")))?;
    Ok(libro.sheet_names().to_vec())
}

fn celda_a_texto(valor: &Data) -> String {
    match valor {
        Data::Empty => String::new(),
        Data::String(s) => s.clone(),
        Data::Float(f) => {
            if f.fract() == 0.0 {
                format!("{}", *f as i64)
            } else {
                f.to_string()
            }
        }
        Data::Int(i) => i.to_string(),
        Data::Bool(b) => b.to_string(),
        Data::DateTime(dt) => dt.to_string(),
        Data::DateTimeIso(s) | Data::DurationIso(s) => s.clone(),
        Data::Error(e) => format!("#ERROR:{e:?}"),
    }
}

pub fn leer_xlsx(ruta: &str, hoja: &str) -> AppResult<ArchivoLeido> {
    let mut libro = open_workbook_auto(ruta)
        .map_err(|e| AppError::Validacion(format!("No se pudo abrir el archivo: {e}")))?;
    let rango = libro
        .worksheet_range(hoja)
        .map_err(|e| AppError::Validacion(format!("No se pudo leer la hoja \"{hoja}\": {e}")))?;

    let mut filas_iter = rango.rows();
    let encabezados = match filas_iter.next() {
        Some(fila) => fila.iter().map(celda_a_texto).collect(),
        None => {
            return Ok(ArchivoLeido {
                encabezados: Vec::new(),
                filas: Vec::new(),
            })
        }
    };
    let filas = filas_iter
        .map(|fila| fila.iter().map(celda_a_texto).collect())
        .collect();

    Ok(ArchivoLeido { encabezados, filas })
}

/// Decodifica los bytes como UTF-8 si son válidos; si no, asume Windows-1252
/// (superconjunto de Latin-1, el caso típico de archivos exportados desde
/// Excel/Windows en español).
fn decodificar_bytes(bytes: &[u8]) -> (String, &'static str) {
    match std::str::from_utf8(bytes) {
        Ok(s) => (s.trim_start_matches('\u{feff}').to_string(), "UTF-8"),
        Err(_) => {
            let (texto, _, _) = encoding_rs::WINDOWS_1252.decode(bytes);
            (texto.into_owned(), "Windows-1252")
        }
    }
}

fn detectar_separador(texto: &str) -> char {
    let primera_linea = texto.lines().next().unwrap_or("");
    let comas = primera_linea.matches(',').count();
    let puntos_y_coma = primera_linea.matches(';').count();
    if puntos_y_coma > comas {
        ';'
    } else {
        ','
    }
}

pub fn leer_csv(
    ruta: &str,
    separador_override: Option<char>,
) -> AppResult<(&'static str, char, ArchivoLeido)> {
    let bytes = std::fs::read(ruta)
        .map_err(|e| AppError::Validacion(format!("No se pudo leer el archivo: {e}")))?;
    let (texto, encoding) = decodificar_bytes(&bytes);
    let separador = separador_override.unwrap_or_else(|| detectar_separador(&texto));

    let mut lector = csv::ReaderBuilder::new()
        .delimiter(separador as u8)
        .flexible(true)
        .has_headers(true)
        .from_reader(texto.as_bytes());

    let encabezados: Vec<String> = lector
        .headers()
        .map_err(|e| AppError::Validacion(format!("No se pudo leer el encabezado: {e}")))?
        .iter()
        .map(|s| s.to_string())
        .collect();

    let mut filas = Vec::new();
    for resultado in lector.records() {
        let registro =
            resultado.map_err(|e| AppError::Validacion(format!("Error leyendo una fila: {e}")))?;
        filas.push(registro.iter().map(|s| s.to_string()).collect());
    }

    Ok((encoding, separador, ArchivoLeido { encabezados, filas }))
}

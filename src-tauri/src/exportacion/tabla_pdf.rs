use std::io::BufWriter;

use printpdf::{Line, Mm, PdfDocument, Point};

use crate::error::{AppError, AppResult};
use crate::pdf::fuente::{medir_ancho_mm, FUENTE_NEGRITA_BYTES, FUENTE_REGULAR_BYTES};

const PT_A_MM: f32 = 25.4 / 72.0;
const ANCHO_MM: f32 = 279.4; // Carta apaisada: entran más columnas que en vertical.
const ALTO_MM: f32 = 215.9;
const MARGEN_MM: f32 = 12.0;
const TAMANO_TITULO_PT: f32 = 14.0;
const TAMANO_CELDA_PT: f32 = 8.5;

/// Recorta el texto agregando "…" hasta que quepa en `ancho_max_mm`.
fn recortar_a_ancho(texto: &str, ancho_max_mm: f32, tamano_pt: f32, negrita: bool) -> String {
    if medir_ancho_mm(texto, tamano_pt, negrita) <= ancho_max_mm {
        return texto.to_string();
    }
    let mut recortado: Vec<char> = texto.chars().collect();
    while !recortado.is_empty() {
        recortado.pop();
        let candidato: String = recortado.iter().collect::<String>() + "…";
        if medir_ancho_mm(&candidato, tamano_pt, negrita) <= ancho_max_mm {
            return candidato;
        }
    }
    "…".to_string()
}

pub fn generar(titulo: &str, columnas: &[String], filas: &[Vec<String>]) -> AppResult<Vec<u8>> {
    let (doc, pagina_inicial, capa_inicial) =
        PdfDocument::new(titulo, Mm(ANCHO_MM), Mm(ALTO_MM), "Capa 1");
    let fuente_regular = doc
        .add_external_font(FUENTE_REGULAR_BYTES)
        .map_err(|e| AppError::Interno(format!("Error cargando fuente del PDF: {e}")))?;
    let fuente_negrita = doc
        .add_external_font(FUENTE_NEGRITA_BYTES)
        .map_err(|e| AppError::Interno(format!("Error cargando fuente del PDF: {e}")))?;

    let mut pagina = pagina_inicial;
    let mut capa = capa_inicial;
    let mut y = ALTO_MM - MARGEN_MM;

    // Título
    {
        let hoja = doc.get_page(pagina).get_layer(capa);
        hoja.use_text(
            titulo,
            TAMANO_TITULO_PT,
            Mm(MARGEN_MM),
            Mm(y),
            &fuente_negrita,
        );
    }
    y -= TAMANO_TITULO_PT * PT_A_MM * 2.0;

    let num_columnas = columnas.len().max(1);
    let ancho_util = ANCHO_MM - MARGEN_MM * 2.0;
    let ancho_col = ancho_util / num_columnas as f32;
    let alto_linea = TAMANO_CELDA_PT * PT_A_MM * 1.7;

    let dibujar_fila = |doc: &printpdf::PdfDocumentReference,
                        pagina: printpdf::PdfPageIndex,
                        capa: printpdf::PdfLayerIndex,
                        y: f32,
                        valores: &[String],
                        negrita: bool| {
        let hoja = doc.get_page(pagina).get_layer(capa);
        let fuente = if negrita {
            &fuente_negrita
        } else {
            &fuente_regular
        };
        for (col, valor) in valores.iter().enumerate() {
            let x = MARGEN_MM + ancho_col * col as f32;
            // Deja un gutter de sobra entre columnas: `recortar_a_ancho` mide con
            // ab_glyph, que no coincide al 100% con las métricas que usa printpdf
            // al dibujar, así que un margen chico no alcanza para evitar que las
            // columnas se toquen.
            let recortado = recortar_a_ancho(valor, ancho_col - 6.0, TAMANO_CELDA_PT, negrita);
            hoja.use_text(&recortado, TAMANO_CELDA_PT, Mm(x), Mm(y), fuente);
        }
    };

    dibujar_fila(&doc, pagina, capa, y, columnas, true);
    y -= alto_linea * 0.5;
    {
        let hoja = doc.get_page(pagina).get_layer(capa);
        let linea = Line {
            points: vec![
                (Point::new(Mm(MARGEN_MM), Mm(y)), false),
                (Point::new(Mm(ANCHO_MM - MARGEN_MM), Mm(y)), false),
            ],
            is_closed: false,
        };
        hoja.add_line(linea);
    }
    y -= alto_linea * 0.7;

    for fila in filas {
        if y - alto_linea < MARGEN_MM {
            let (p, c) = doc.add_page(Mm(ANCHO_MM), Mm(ALTO_MM), "Capa 1");
            pagina = p;
            capa = c;
            y = ALTO_MM - MARGEN_MM;
        }
        dibujar_fila(&doc, pagina, capa, y, fila, false);
        y -= alto_linea;
    }

    let mut buffer = Vec::new();
    {
        let mut writer = BufWriter::new(&mut buffer);
        doc.save(&mut writer)
            .map_err(|e| AppError::Interno(format!("Error guardando el PDF: {e}")))?;
    }
    Ok(buffer)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recorta_texto_largo_con_elipsis_y_respeta_el_ancho() {
        let texto = "Producto de ejemplo con nombre bastante largo número 1";
        let recortado = recortar_a_ancho(texto, 45.0, TAMANO_CELDA_PT, false);
        assert!(recortado.ends_with('…'));
        assert!(recortado.len() < texto.len());
        assert!(medir_ancho_mm(&recortado, TAMANO_CELDA_PT, false) <= 45.0);
    }

    #[test]
    fn no_recorta_texto_que_ya_entra() {
        let recortado = recortar_a_ancho("Corto", 100.0, TAMANO_CELDA_PT, false);
        assert_eq!(recortado, "Corto");
    }

    #[test]
    fn genera_pdf_con_muchas_filas_y_pagina() {
        let columnas = vec!["Producto".to_string(), "Cantidad".to_string()];
        let filas: Vec<Vec<String>> = (0..80)
            .map(|i| vec![format!("Producto {i}"), format!("{i}")])
            .collect();
        let bytes = generar("Reporte largo", &columnas, &filas).unwrap();
        assert!(!bytes.is_empty());
        assert_eq!(&bytes[0..5], b"%PDF-");
    }
}

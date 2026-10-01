use ab_glyph::PxScale;
use image::{Rgba, RgbaImage};
use imageproc::drawing::{draw_line_segment_mut, draw_text_mut, text_size};

use crate::error::AppError;
use crate::error::AppResult;
use crate::pdf::elementos::{Alineacion, Elemento};
use crate::pdf::fuente::{fuente_negrita, fuente_regular};

const ANCHO_MM: f32 = 80.0;
const MARGEN_MM: f32 = 4.0;
const TAMANO_BASE_PT: f32 = 8.0;
/// Densidad de la imagen: píxeles por milímetro (~102 dpi), suficiente para
/// verse nítido al enviarlo por WhatsApp/correo.
const PX_POR_MM: f32 = 4.0;
const PT_A_PX: f32 = (25.4 / 72.0) * PX_POR_MM;

pub fn renderizar(elementos: &[Elemento], logo_path: Option<&str>) -> AppResult<Vec<u8>> {
    let logo = logo_path.and_then(|ruta| match image::open(ruta) {
        Ok(img) => Some(img),
        Err(e) => {
            tracing::warn!("No se pudo cargar el logo desde {ruta}: {e}");
            None
        }
    });

    let ancho_px = (ANCHO_MM * PX_POR_MM) as u32;
    let margen_px = MARGEN_MM * PX_POR_MM;
    let ancho_objetivo_logo_px = (30.0 * PX_POR_MM).min(ancho_px as f32 - 2.0 * margen_px);
    let alto_logo_px = logo.as_ref().map(|img| {
        let factor = ancho_objetivo_logo_px / img.width() as f32;
        img.height() as f32 * factor
    });

    let alto_mm = calcular_alto_mm(elementos);
    let alto_px_contenido = (alto_mm * PX_POR_MM) as u32;
    let alto_px = alto_px_contenido + alto_logo_px.map(|a| a as u32 + 8).unwrap_or(0);

    let mut imagen = RgbaImage::from_pixel(ancho_px, alto_px, Rgba([255, 255, 255, 255]));
    let negro = Rgba([25, 25, 25, 255]);

    let fuente_reg = fuente_regular();
    let fuente_neg = fuente_negrita();
    let ancho_px_f = ancho_px as f32;

    let mut y = margen_px;

    if let (Some(img), Some(alto_logo)) = (logo, alto_logo_px) {
        let logo_redimensionado = img.resize(
            ancho_objetivo_logo_px as u32,
            alto_logo as u32,
            image::imageops::FilterType::Lanczos3,
        );
        let x = ((ancho_px_f - ancho_objetivo_logo_px) / 2.0) as i64;
        image::imageops::overlay(&mut imagen, &logo_redimensionado.to_rgba8(), x, y as i64);
        y += alto_logo + 8.0;
    }
    for el in elementos {
        match el {
            Elemento::Texto {
                texto,
                alineacion,
                negrita,
                delta_tamano,
            } => {
                let tamano_pt = (TAMANO_BASE_PT + delta_tamano).max(5.0);
                let escala = PxScale::from(tamano_pt * PT_A_PX);
                let fuente = if *negrita { &fuente_neg } else { &fuente_reg };
                let (ancho_texto, _) = text_size(escala, fuente, texto);
                let x = match alineacion {
                    Alineacion::Izquierda => margen_px,
                    Alineacion::Centro => (ancho_px_f - ancho_texto as f32) / 2.0,
                };
                draw_text_mut(
                    &mut imagen,
                    negro,
                    x as i32,
                    y as i32,
                    escala,
                    fuente,
                    texto,
                );
                y += tamano_pt * PT_A_PX * 1.5;
            }
            Elemento::DosColumnas {
                izquierda,
                derecha,
                negrita,
            } => {
                let escala = PxScale::from(TAMANO_BASE_PT * PT_A_PX);
                let fuente = if *negrita { &fuente_neg } else { &fuente_reg };
                draw_text_mut(
                    &mut imagen,
                    negro,
                    margen_px as i32,
                    y as i32,
                    escala,
                    fuente,
                    izquierda,
                );
                let (ancho_derecha, _) = text_size(escala, fuente, derecha);
                let x_derecha = ancho_px_f - margen_px - ancho_derecha as f32;
                draw_text_mut(
                    &mut imagen,
                    negro,
                    x_derecha as i32,
                    y as i32,
                    escala,
                    fuente,
                    derecha,
                );
                y += TAMANO_BASE_PT * PT_A_PX * 1.5;
            }
            Elemento::Separador => {
                let altura_linea = TAMANO_BASE_PT * PT_A_PX * 0.8;
                let ym = y + altura_linea / 2.0;
                draw_line_segment_mut(
                    &mut imagen,
                    (margen_px, ym),
                    (ancho_px_f - margen_px, ym),
                    Rgba([180, 180, 180, 255]),
                );
                y += altura_linea;
            }
            Elemento::Espacio => {
                y += TAMANO_BASE_PT * PT_A_PX;
            }
        }
    }

    let mut buffer = Vec::new();
    let mut cursor = std::io::Cursor::new(&mut buffer);
    image::DynamicImage::ImageRgba8(imagen)
        .write_to(&mut cursor, image::ImageFormat::Png)
        .map_err(|e| AppError::Interno(format!("Error codificando la imagen: {e}")))?;
    Ok(buffer)
}

fn calcular_alto_mm(elementos: &[Elemento]) -> f32 {
    let mut alto = MARGEN_MM * 2.0;
    for el in elementos {
        alto += match el {
            Elemento::Texto { delta_tamano, .. } => {
                (TAMANO_BASE_PT + delta_tamano).max(5.0) * (25.4 / 72.0) * 1.5
            }
            Elemento::DosColumnas { .. } => TAMANO_BASE_PT * (25.4 / 72.0) * 1.5,
            Elemento::Separador => TAMANO_BASE_PT * (25.4 / 72.0) * 0.8,
            Elemento::Espacio => TAMANO_BASE_PT * (25.4 / 72.0),
        };
    }
    alto
}

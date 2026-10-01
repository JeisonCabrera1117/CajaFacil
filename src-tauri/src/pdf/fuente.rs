use ab_glyph::{Font, FontRef, PxScale, ScaleFont};

pub static FUENTE_REGULAR_BYTES: &[u8] = include_bytes!("../../assets/fonts/DejaVuSans.ttf");
pub static FUENTE_NEGRITA_BYTES: &[u8] = include_bytes!("../../assets/fonts/DejaVuSans-Bold.ttf");

pub fn fuente_regular() -> FontRef<'static> {
    FontRef::try_from_slice(FUENTE_REGULAR_BYTES).expect("fuente regular embebida inválida")
}

pub fn fuente_negrita() -> FontRef<'static> {
    FontRef::try_from_slice(FUENTE_NEGRITA_BYTES).expect("fuente negrita embebida inválida")
}

const PT_A_MM: f32 = 25.4 / 72.0;

/// Ancho del texto en milímetros para un tamaño de fuente en puntos, medido
/// con las métricas reales de la fuente embebida (no un promedio por carácter).
pub fn medir_ancho_mm(texto: &str, tamano_pt: f32, negrita: bool) -> f32 {
    let fuente = if negrita {
        fuente_negrita()
    } else {
        fuente_regular()
    };
    let escalada = fuente.as_scaled(PxScale::from(tamano_pt));
    let ancho_pt: f32 = texto
        .chars()
        .map(|c| escalada.h_advance(fuente.glyph_id(c)))
        .sum();
    ancho_pt * PT_A_MM
}

use std::io::BufWriter;

use printpdf::{
    Image, ImageTransform, IndirectFontRef, Line, Mm, PdfDocument, PdfDocumentReference,
    PdfLayerIndex, PdfPageIndex, Point,
};

use crate::error::{AppError, AppResult};
use crate::pdf::elementos::{Alineacion, Elemento};
use crate::pdf::fuente::{medir_ancho_mm, FUENTE_NEGRITA_BYTES, FUENTE_REGULAR_BYTES};

const PT_A_MM: f32 = 25.4 / 72.0;

struct Lienzo {
    doc: PdfDocumentReference,
    fuente_regular: IndirectFontRef,
    fuente_negrita: IndirectFontRef,
    ancho_mm: f32,
    alto_mm: f32,
    margen_mm: f32,
    tamano_base_pt: f32,
    pagina: PdfPageIndex,
    capa: PdfLayerIndex,
    y: f32,
}

impl Lienzo {
    fn nuevo(ancho_mm: f32, alto_mm: f32, margen_mm: f32, tamano_base_pt: f32) -> AppResult<Self> {
        let (doc, pagina, capa) =
            PdfDocument::new("Comprobante CajaFácil", Mm(ancho_mm), Mm(alto_mm), "Capa 1");
        let fuente_regular = doc
            .add_external_font(FUENTE_REGULAR_BYTES)
            .map_err(|e| AppError::Interno(format!("Error cargando fuente del PDF: {e}")))?;
        let fuente_negrita = doc
            .add_external_font(FUENTE_NEGRITA_BYTES)
            .map_err(|e| AppError::Interno(format!("Error cargando fuente del PDF: {e}")))?;
        Ok(Self {
            doc,
            fuente_regular,
            fuente_negrita,
            ancho_mm,
            alto_mm,
            margen_mm,
            tamano_base_pt,
            pagina,
            capa,
            y: alto_mm - margen_mm,
        })
    }

    fn asegurar_espacio(&mut self, alto_linea_mm: f32) {
        if self.y - alto_linea_mm < self.margen_mm {
            let (pagina, capa) = self
                .doc
                .add_page(Mm(self.ancho_mm), Mm(self.alto_mm), "Capa 1");
            self.pagina = pagina;
            self.capa = capa;
            self.y = self.alto_mm - self.margen_mm;
        }
    }

    fn texto(&mut self, texto: &str, x_mm: f32, tamano_pt: f32, negrita: bool) {
        let capa = self.doc.get_page(self.pagina).get_layer(self.capa);
        let fuente = if negrita {
            &self.fuente_negrita
        } else {
            &self.fuente_regular
        };
        capa.use_text(texto, tamano_pt, Mm(x_mm), Mm(self.y), fuente);
    }

    fn linea_horizontal(&mut self) {
        let capa = self.doc.get_page(self.pagina).get_layer(self.capa);
        let linea = Line {
            points: vec![
                (Point::new(Mm(self.margen_mm), Mm(self.y)), false),
                (
                    Point::new(Mm(self.ancho_mm - self.margen_mm), Mm(self.y)),
                    false,
                ),
            ],
            is_closed: false,
        };
        capa.add_line(linea);
    }

    fn dibujar(&mut self, elementos: &[Elemento]) {
        for el in elementos {
            match el {
                Elemento::Texto {
                    texto: contenido,
                    alineacion,
                    negrita,
                    delta_tamano,
                } => {
                    let tamano = (self.tamano_base_pt + delta_tamano).max(5.0);
                    let alto_linea = tamano * PT_A_MM * 1.5;
                    self.asegurar_espacio(alto_linea);
                    let x = match alineacion {
                        Alineacion::Izquierda => self.margen_mm,
                        Alineacion::Centro => {
                            let ancho_texto = medir_ancho_mm(contenido, tamano, *negrita);
                            (self.ancho_mm - ancho_texto) / 2.0
                        }
                    };
                    self.texto(contenido, x, tamano, *negrita);
                    self.y -= alto_linea;
                }
                Elemento::DosColumnas {
                    izquierda,
                    derecha,
                    negrita,
                } => {
                    let tamano = self.tamano_base_pt;
                    let alto_linea = tamano * PT_A_MM * 1.5;
                    self.asegurar_espacio(alto_linea);
                    self.texto(izquierda, self.margen_mm, tamano, *negrita);
                    let ancho_derecha = medir_ancho_mm(derecha, tamano, *negrita);
                    let x_derecha = self.ancho_mm - self.margen_mm - ancho_derecha;
                    self.texto(derecha, x_derecha, tamano, *negrita);
                    self.y -= alto_linea;
                }
                Elemento::Separador => {
                    // `use_text` posiciona por línea base, no por el tope del glifo: hay que
                    // dejar espacio de sobra antes Y después de la línea, si no el trazo queda
                    // encima del texto de arriba o cruzando los ascendentes del texto de abajo.
                    let paso = self.tamano_base_pt * PT_A_MM;
                    self.asegurar_espacio(paso * 1.5);
                    self.y -= paso * 0.5;
                    self.linea_horizontal();
                    self.y -= paso;
                }
                Elemento::Espacio => {
                    self.y -= self.tamano_base_pt * PT_A_MM;
                }
            }
        }
    }

    /// `printpdf::Image::from_dynamic_image` exige la versión exacta de
    /// `image` que printpdf vendoriza por debajo (0.24.9) — por eso se carga
    /// con el crate renombrado `image-para-printpdf` en vez del `image` 0.25
    /// que usa el resto del código (ver comentario en `Cargo.toml`).
    fn dibujar_logo(&mut self, logo_path: &str) {
        let logo = match image_para_printpdf::open(logo_path) {
            Ok(img) => img,
            Err(e) => {
                tracing::warn!("No se pudo cargar el logo desde {logo_path}: {e}");
                return;
            }
        };

        let ancho_objetivo_mm = 30.0_f32.min(self.ancho_mm - 2.0 * self.margen_mm).max(10.0);
        let dpi = 300.0_f32;
        let ancho_nativo_mm = (logo.width() as f32 / dpi) * 25.4;
        if ancho_nativo_mm <= 0.0 {
            return;
        }
        let alto_nativo_mm = (logo.height() as f32 / dpi) * 25.4;
        let factor = ancho_objetivo_mm / ancho_nativo_mm;
        let alto_final_mm = alto_nativo_mm * factor;

        self.asegurar_espacio(alto_final_mm);
        let x = (self.ancho_mm - ancho_objetivo_mm) / 2.0;
        let y_base = self.y - alto_final_mm;

        let imagen_pdf = Image::from_dynamic_image(&logo);
        let capa = self.doc.get_page(self.pagina).get_layer(self.capa);
        imagen_pdf.add_to_layer(
            capa,
            ImageTransform {
                translate_x: Some(Mm(x)),
                translate_y: Some(Mm(y_base)),
                scale_x: Some(factor as f32),
                scale_y: Some(factor as f32),
                dpi: Some(dpi),
                ..Default::default()
            },
        );
        self.y = y_base - 2.0;
    }

    fn guardar(self) -> AppResult<Vec<u8>> {
        let mut buffer = Vec::new();
        {
            let mut writer = BufWriter::new(&mut buffer);
            self.doc
                .save(&mut writer)
                .map_err(|e| AppError::Interno(format!("Error guardando el PDF: {e}")))?;
        }
        Ok(buffer)
    }
}

pub fn renderizar_carta(elementos: &[Elemento], logo_path: Option<&str>) -> AppResult<Vec<u8>> {
    let mut lienzo = Lienzo::nuevo(215.9, 279.4, 18.0, 11.0)?;
    if let Some(ruta) = logo_path {
        lienzo.dibujar_logo(ruta);
    }
    lienzo.dibujar(elementos);
    lienzo.guardar()
}

pub fn renderizar_termico(elementos: &[Elemento], logo_path: Option<&str>) -> AppResult<Vec<u8>> {
    let alto_extra_logo = if logo_path.is_some() { 35.0 } else { 0.0 };
    let alto_estimado = calcular_alto_estimado_mm(elementos, 8.0) + alto_extra_logo;
    let mut lienzo = Lienzo::nuevo(80.0, alto_estimado, 4.0, 8.0)?;
    if let Some(ruta) = logo_path {
        lienzo.dibujar_logo(ruta);
    }
    lienzo.dibujar(elementos);
    lienzo.guardar()
}

fn calcular_alto_estimado_mm(elementos: &[Elemento], tamano_base_pt: f32) -> f32 {
    let margen = 8.0;
    let mut alto = margen * 2.0;
    for el in elementos {
        alto += match el {
            Elemento::Texto { delta_tamano, .. } => {
                (tamano_base_pt + delta_tamano).max(5.0) * PT_A_MM * 1.5
            }
            Elemento::DosColumnas { .. } => tamano_base_pt * PT_A_MM * 1.5,
            Elemento::Separador => tamano_base_pt * PT_A_MM * 0.8,
            Elemento::Espacio => tamano_base_pt * PT_A_MM,
        };
    }
    alto
}

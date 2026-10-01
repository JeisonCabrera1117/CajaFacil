use crate::pdf::datos::{formatear_fecha, formatear_moneda, DatosComprobante};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Alineacion {
    Izquierda,
    Centro,
}

/// Modelo de documento independiente del renderer: tanto el PDF (carta y
/// térmico) como la imagen PNG se construyen recorriendo la misma lista.
pub enum Elemento {
    Texto {
        texto: String,
        alineacion: Alineacion,
        negrita: bool,
        delta_tamano: f32,
    },
    DosColumnas {
        izquierda: String,
        derecha: String,
        negrita: bool,
    },
    Separador,
    Espacio,
}

fn texto(texto: impl Into<String>) -> Elemento {
    Elemento::Texto {
        texto: texto.into(),
        alineacion: Alineacion::Izquierda,
        negrita: false,
        delta_tamano: 0.0,
    }
}

fn texto_centro(texto: impl Into<String>, negrita: bool, delta_tamano: f32) -> Elemento {
    Elemento::Texto {
        texto: texto.into(),
        alineacion: Alineacion::Centro,
        negrita,
        delta_tamano,
    }
}

fn dos_columnas(
    izquierda: impl Into<String>,
    derecha: impl Into<String>,
    negrita: bool,
) -> Elemento {
    Elemento::DosColumnas {
        izquierda: izquierda.into(),
        derecha: derecha.into(),
        negrita,
    }
}

pub fn construir(datos: &DatosComprobante) -> Vec<Elemento> {
    let moneda = datos.moneda.as_str();
    let mut el = Vec::new();

    el.push(texto_centro(&datos.empresa_nombre, true, 3.0));
    if !datos.empresa_nit.trim().is_empty() {
        el.push(texto_centro(
            format!("NIT: {}", datos.empresa_nit),
            false,
            0.0,
        ));
    }
    if !datos.empresa_direccion.trim().is_empty() {
        el.push(texto_centro(&datos.empresa_direccion, false, 0.0));
    }
    if !datos.empresa_telefono.trim().is_empty() {
        el.push(texto_centro(
            format!("Tel: {}", datos.empresa_telefono),
            false,
            0.0,
        ));
    }
    el.push(Elemento::Separador);

    let venta = &datos.venta.venta;
    el.push(texto(format!("Comprobante: {}", venta.numero_comprobante)));
    el.push(texto(formatear_fecha(&venta.fecha, &datos.formato_fecha)));
    el.push(texto(format!(
        "Cliente: {}",
        venta.cliente_nombre.as_deref().unwrap_or("Cliente general")
    )));
    el.push(Elemento::Separador);

    for item in &datos.venta.items {
        el.push(texto(&item.producto_nombre));
        let detalle = format!(
            "  {} x {}",
            item.cantidad,
            formatear_moneda(item.precio_unitario, moneda)
        );
        el.push(dos_columnas(
            detalle,
            formatear_moneda(item.subtotal, moneda),
            false,
        ));
        if item.descuento_valor > 0 {
            el.push(Elemento::Texto {
                texto: format!(
                    "  Descuento: -{}",
                    formatear_moneda(item.descuento_valor, moneda)
                ),
                alineacion: Alineacion::Izquierda,
                negrita: false,
                delta_tamano: -1.0,
            });
        }
    }
    el.push(Elemento::Separador);

    el.push(dos_columnas(
        "Subtotal",
        formatear_moneda(venta.subtotal, moneda),
        false,
    ));
    if venta.descuento_total > 0 {
        el.push(dos_columnas(
            "Descuentos",
            format!("-{}", formatear_moneda(venta.descuento_total, moneda)),
            false,
        ));
    }
    if venta.impuestos > 0 {
        el.push(dos_columnas(
            "Impuestos",
            formatear_moneda(venta.impuestos, moneda),
            false,
        ));
    }
    el.push(dos_columnas(
        "TOTAL",
        formatear_moneda(venta.total, moneda),
        true,
    ));
    el.push(Elemento::Espacio);

    el.push(texto(format!(
        "Método de pago: {}",
        capitalizar(&venta.metodo_pago)
    )));
    if let Some(recibido) = venta.valor_recibido {
        el.push(dos_columnas(
            "Recibido",
            formatear_moneda(recibido, moneda),
            false,
        ));
    }
    if let Some(cambio) = venta.cambio {
        el.push(dos_columnas(
            "Cambio",
            formatear_moneda(cambio, moneda),
            false,
        ));
    }
    if venta.estado == "anulada" {
        el.push(Elemento::Espacio);
        el.push(texto_centro("*** VENTA ANULADA ***", true, 0.0));
        if let Some(motivo) = &venta.motivo_anulacion {
            el.push(texto_centro(format!("Motivo: {motivo}"), false, -1.0));
        }
    }

    el.push(Elemento::Separador);
    if !datos.leyenda_pie.trim().is_empty() {
        el.push(texto_centro(&datos.leyenda_pie, false, -1.0));
    }

    el
}

fn capitalizar(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        Some(primera) => primera.to_uppercase().collect::<String>() + c.as_str(),
        None => String::new(),
    }
}

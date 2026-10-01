/// Interpreta números escritos en formato colombiano ("1.234,56") o estándar
/// ("1234.56" / "1,234.56"), sin ambigüedad: si aparecen los dos separadores,
/// el que aparece de último es el decimal. Si aparece solo uno y tiene 1-2
/// dígitos después, se asume decimal; si tiene 3+ dígitos después, se asume
/// separador de miles.
pub fn parsear_numero(texto: &str) -> Option<f64> {
    let t = texto.trim();
    if t.is_empty() {
        return None;
    }
    let tiene_coma = t.contains(',');
    let tiene_punto = t.contains('.');

    let normalizado = if tiene_coma && tiene_punto {
        let pos_coma = t.rfind(',').unwrap();
        let pos_punto = t.rfind('.').unwrap();
        if pos_coma > pos_punto {
            t.replace('.', "").replace(',', ".")
        } else {
            t.replace(',', "")
        }
    } else if tiene_coma {
        let pos = t.rfind(',').unwrap();
        let digitos_despues = t.len() - pos - 1;
        if digitos_despues <= 2 {
            t.replace(',', ".")
        } else {
            t.replace(',', "")
        }
    } else if tiene_punto {
        let pos = t.rfind('.').unwrap();
        let digitos_despues = t.len() - pos - 1;
        if digitos_despues <= 2 {
            t.to_string()
        } else {
            t.replace('.', "")
        }
    } else {
        t.to_string()
    };

    normalizado.parse::<f64>().ok()
}

pub fn parsear_entero(texto: &str) -> Option<i64> {
    parsear_numero(texto).map(|n| n.round() as i64)
}

/// Convierte a centavos (para los montos que la BD guarda como enteros).
pub fn parsear_centavos(texto: &str) -> Option<i64> {
    parsear_numero(texto).map(|n| (n * 100.0).round() as i64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formato_colombiano() {
        assert_eq!(parsear_numero("1.234,56"), Some(1234.56));
        assert_eq!(parsear_numero("1.234"), Some(1234.0));
        assert_eq!(parsear_numero("12,5"), Some(12.5));
    }

    #[test]
    fn formato_estandar() {
        assert_eq!(parsear_numero("1234.56"), Some(1234.56));
        assert_eq!(parsear_numero("1,234.56"), Some(1234.56));
        assert_eq!(parsear_numero("1234"), Some(1234.0));
    }

    #[test]
    fn miles_sin_decimales() {
        assert_eq!(parsear_numero("1,234"), Some(1234.0));
        assert_eq!(parsear_numero("12.345"), Some(12345.0));
    }

    #[test]
    fn invalido_o_vacio() {
        assert_eq!(parsear_numero(""), None);
        assert_eq!(parsear_numero("   "), None);
        assert_eq!(parsear_numero("abc"), None);
    }

    #[test]
    fn centavos_redondea() {
        assert_eq!(parsear_centavos("10"), Some(1000));
        assert_eq!(parsear_centavos("10,5"), Some(1050));
    }
}

use std::collections::HashMap;

use rusqlite::{params, Connection, OptionalExtension};

use crate::importacion::ayudas::{texto, traducir_error_sql};
use crate::importacion::ResultadoFila;
use crate::models::importacion::{CampoImport, ModoImport};

pub fn campos() -> Vec<CampoImport> {
    vec![
        CampoImport::opcional("nit", "NIT"),
        CampoImport::obligatorio("razonSocial", "Razón social"),
        CampoImport::opcional("contacto", "Contacto"),
        CampoImport::opcional("telefono", "Teléfono"),
        CampoImport::opcional("correo", "Correo"),
        CampoImport::opcional("direccion", "Dirección"),
        CampoImport::opcional("condicionesPago", "Condiciones de pago"),
        CampoImport::opcional("activo", "Activo (si/no)"),
    ]
}

fn es_afirmativo(valor: &str) -> bool {
    matches!(
        valor.trim().to_lowercase().as_str(),
        "si" | "sí" | "true" | "1" | "yes"
    )
}

/// Clave de emparejamiento: NIT si viene informado, si no la razón social.
pub fn procesar_fila(
    tx: &Connection,
    fila: &HashMap<String, String>,
    modo: ModoImport,
) -> ResultadoFila {
    let Some(razon_social) = texto(fila, "razonSocial") else {
        return ResultadoFila::Error("La razón social es obligatoria.".into());
    };
    let nit = texto(fila, "nit");
    let contacto = texto(fila, "contacto");
    let telefono = texto(fila, "telefono");
    let correo = texto(fila, "correo");
    let direccion = texto(fila, "direccion");
    let condiciones_pago = texto(fila, "condicionesPago");
    let activo = texto(fila, "activo")
        .map(|s| es_afirmativo(&s))
        .unwrap_or(true);

    let existente: Option<i64> = if let Some(nit_valor) = &nit {
        tx.query_row(
            "SELECT id FROM proveedores WHERE nit = ?1",
            [nit_valor],
            |r| r.get(0),
        )
        .optional()
    } else {
        tx.query_row(
            "SELECT id FROM proveedores WHERE razon_social = ?1",
            [&razon_social],
            |r| r.get(0),
        )
        .optional()
    }
    .unwrap_or(None);

    match (existente, modo) {
        (Some(_), ModoImport::Crear) => ResultadoFila::Omitido("Ya existe un proveedor con esa clave.".into()),
        (None, ModoImport::Actualizar) => {
            ResultadoFila::Omitido("No existe un proveedor con esa clave para actualizar.".into())
        }
        (Some(id), ModoImport::Actualizar) | (Some(id), ModoImport::CrearYActualizar) => {
            match tx.execute(
                "UPDATE proveedores SET nit=?1, razon_social=?2, contacto=?3, telefono=?4, correo=?5,
                    direccion=?6, condiciones_pago=?7, activo=?8 WHERE id = ?9",
                params![nit, razon_social, contacto, telefono, correo, direccion, condiciones_pago, activo as i64, id],
            ) {
                Ok(_) => ResultadoFila::Actualizado,
                Err(e) => ResultadoFila::Error(traducir_error_sql(&e)),
            }
        }
        (None, ModoImport::Crear) | (None, ModoImport::CrearYActualizar) => {
            match tx.execute(
                "INSERT INTO proveedores (nit, razon_social, contacto, telefono, correo, direccion, condiciones_pago, activo)
                 VALUES (?1,?2,?3,?4,?5,?6,?7,?8)",
                params![nit, razon_social, contacto, telefono, correo, direccion, condiciones_pago, activo as i64],
            ) {
                Ok(_) => ResultadoFila::Creado,
                Err(e) => ResultadoFila::Error(traducir_error_sql(&e)),
            }
        }
    }
}

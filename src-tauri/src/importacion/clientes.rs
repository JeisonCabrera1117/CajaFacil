use std::collections::HashMap;

use rusqlite::{params, Connection, OptionalExtension};

use crate::importacion::ayudas::{texto, traducir_error_sql};
use crate::importacion::ResultadoFila;
use crate::models::importacion::{CampoImport, ModoImport};

pub fn campos() -> Vec<CampoImport> {
    vec![
        CampoImport::opcional("documento", "Documento"),
        CampoImport::obligatorio("nombre", "Nombre"),
        CampoImport::opcional("telefono", "Teléfono"),
        CampoImport::opcional("correo", "Correo"),
        CampoImport::opcional("direccion", "Dirección"),
    ]
}

/// Clave de emparejamiento: documento si viene informado, si no el nombre.
pub fn procesar_fila(
    tx: &Connection,
    fila: &HashMap<String, String>,
    modo: ModoImport,
) -> ResultadoFila {
    let Some(nombre) = texto(fila, "nombre") else {
        return ResultadoFila::Error("El nombre es obligatorio.".into());
    };
    let documento = texto(fila, "documento");
    let telefono = texto(fila, "telefono");
    let correo = texto(fila, "correo");
    let direccion = texto(fila, "direccion");

    let existente: Option<i64> = if let Some(doc) = &documento {
        tx.query_row("SELECT id FROM clientes WHERE documento = ?1", [doc], |r| {
            r.get(0)
        })
        .optional()
    } else {
        tx.query_row(
            "SELECT id FROM clientes WHERE nombre = ?1",
            [&nombre],
            |r| r.get(0),
        )
        .optional()
    }
    .unwrap_or(None);

    match (existente, modo) {
        (Some(_), ModoImport::Crear) => ResultadoFila::Omitido("Ya existe un cliente con esa clave.".into()),
        (None, ModoImport::Actualizar) => {
            ResultadoFila::Omitido("No existe un cliente con esa clave para actualizar.".into())
        }
        (Some(id), ModoImport::Actualizar) | (Some(id), ModoImport::CrearYActualizar) => {
            match tx.execute(
                "UPDATE clientes SET documento=?1, nombre=?2, telefono=?3, correo=?4, direccion=?5 WHERE id=?6",
                params![documento, nombre, telefono, correo, direccion, id],
            ) {
                Ok(_) => ResultadoFila::Actualizado,
                Err(e) => ResultadoFila::Error(traducir_error_sql(&e)),
            }
        }
        (None, ModoImport::Crear) | (None, ModoImport::CrearYActualizar) => {
            match tx.execute(
                "INSERT INTO clientes (documento, nombre, telefono, correo, direccion) VALUES (?1,?2,?3,?4,?5)",
                params![documento, nombre, telefono, correo, direccion],
            ) {
                Ok(_) => ResultadoFila::Creado,
                Err(e) => ResultadoFila::Error(traducir_error_sql(&e)),
            }
        }
    }
}

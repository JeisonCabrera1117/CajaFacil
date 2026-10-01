use std::collections::HashMap;

use rusqlite::{params, Connection, OptionalExtension};

use crate::importacion::ayudas::{texto, traducir_error_sql};
use crate::importacion::ResultadoFila;
use crate::models::importacion::{CampoImport, ModoImport};

pub fn campos() -> Vec<CampoImport> {
    vec![
        CampoImport::obligatorio("nombre", "Nombre"),
        CampoImport::opcional("categoriaPadre", "Categoría padre (nombre)"),
    ]
}

pub fn procesar_fila(
    tx: &Connection,
    fila: &HashMap<String, String>,
    modo: ModoImport,
) -> ResultadoFila {
    let Some(nombre) = texto(fila, "nombre") else {
        return ResultadoFila::Error("El nombre es obligatorio.".into());
    };

    let padre_id: Option<i64> = match texto(fila, "categoriaPadre") {
        Some(nombre_padre) => {
            if nombre_padre == nombre {
                return ResultadoFila::Error(
                    "Una categoría no puede ser su propia categoría padre.".into(),
                );
            }
            match tx
                .query_row(
                    "SELECT id FROM categorias WHERE nombre = ?1",
                    [&nombre_padre],
                    |r| r.get(0),
                )
                .optional()
            {
                Ok(Some(id)) => Some(id),
                Ok(None) => {
                    return ResultadoFila::Error(format!(
                        "La categoría padre \"{nombre_padre}\" no existe."
                    ))
                }
                Err(e) => return ResultadoFila::Error(traducir_error_sql(&e)),
            }
        }
        None => None,
    };

    let existente: Option<i64> = match tx
        .query_row(
            "SELECT id FROM categorias WHERE nombre = ?1",
            [&nombre],
            |r| r.get(0),
        )
        .optional()
    {
        Ok(v) => v,
        Err(e) => return ResultadoFila::Error(traducir_error_sql(&e)),
    };

    match (existente, modo) {
        (Some(_), ModoImport::Crear) => {
            ResultadoFila::Omitido("Ya existe una categoría con ese nombre.".into())
        }
        (None, ModoImport::Actualizar) => {
            ResultadoFila::Omitido("No existe una categoría con ese nombre para actualizar.".into())
        }
        (Some(id), ModoImport::Actualizar) | (Some(id), ModoImport::CrearYActualizar) => {
            match tx.execute(
                "UPDATE categorias SET categoria_padre_id = ?1 WHERE id = ?2",
                params![padre_id, id],
            ) {
                Ok(_) => ResultadoFila::Actualizado,
                Err(e) => ResultadoFila::Error(traducir_error_sql(&e)),
            }
        }
        (None, ModoImport::Crear) | (None, ModoImport::CrearYActualizar) => {
            match tx.execute(
                "INSERT INTO categorias (nombre, categoria_padre_id) VALUES (?1, ?2)",
                params![nombre, padre_id],
            ) {
                Ok(_) => ResultadoFila::Creado,
                Err(e) => ResultadoFila::Error(traducir_error_sql(&e)),
            }
        }
    }
}

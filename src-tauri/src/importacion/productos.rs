use std::collections::HashMap;

use rusqlite::{params, Connection, OptionalExtension};

use crate::importacion::ayudas::{
    obtener_o_crear_categoria, obtener_o_crear_proveedor, texto, traducir_error_sql,
};
use crate::importacion::numeros;
use crate::importacion::ResultadoFila;
use crate::models::importacion::{CampoImport, ModoImport};

pub fn campos() -> Vec<CampoImport> {
    vec![
        CampoImport::obligatorio("sku", "SKU"),
        CampoImport::opcional("codigoBarras", "Código de barras"),
        CampoImport::obligatorio("nombre", "Nombre"),
        CampoImport::opcional("descripcion", "Descripción"),
        CampoImport::opcional("categoria", "Categoría (nombre)"),
        CampoImport::opcional("unidadMedida", "Unidad de medida"),
        CampoImport::obligatorio("precioCosto", "Precio de costo"),
        CampoImport::obligatorio("precioVenta", "Precio de venta"),
        CampoImport::opcional("impuestoPct", "Impuesto (%)"),
        CampoImport::opcional("stockMinimo", "Stock mínimo"),
        CampoImport::opcional("stockMaximo", "Stock máximo"),
        CampoImport::opcional("ubicacion", "Ubicación"),
        CampoImport::opcional("proveedorPrincipal", "Proveedor principal (razón social)"),
        CampoImport::opcional("estado", "Estado (activo/inactivo)"),
    ]
}

pub fn procesar_fila(
    tx: &Connection,
    fila: &HashMap<String, String>,
    modo: ModoImport,
) -> ResultadoFila {
    let Some(sku) = texto(fila, "sku") else {
        return ResultadoFila::Error("El SKU es obligatorio.".into());
    };
    let Some(nombre) = texto(fila, "nombre") else {
        return ResultadoFila::Error("El nombre es obligatorio.".into());
    };
    let Some(precio_costo) = fila
        .get("precioCosto")
        .and_then(|s| numeros::parsear_centavos(s))
    else {
        return ResultadoFila::Error(
            "El precio de costo es obligatorio y debe ser numérico.".into(),
        );
    };
    let Some(precio_venta) = fila
        .get("precioVenta")
        .and_then(|s| numeros::parsear_centavos(s))
    else {
        return ResultadoFila::Error(
            "El precio de venta es obligatorio y debe ser numérico.".into(),
        );
    };

    let impuesto_pct = fila
        .get("impuestoPct")
        .and_then(|s| numeros::parsear_numero(s))
        .unwrap_or(0.0);
    let stock_minimo = fila
        .get("stockMinimo")
        .and_then(|s| numeros::parsear_entero(s))
        .unwrap_or(0);
    let stock_maximo = fila
        .get("stockMaximo")
        .and_then(|s| numeros::parsear_entero(s));
    let ubicacion = texto(fila, "ubicacion");
    let codigo_barras = texto(fila, "codigoBarras");
    let descripcion = texto(fila, "descripcion");
    let unidad_medida = texto(fila, "unidadMedida").unwrap_or_else(|| "unidad".into());
    let estado = if texto(fila, "estado").map(|s| s.to_lowercase()) == Some("inactivo".into()) {
        "inactivo"
    } else {
        "activo"
    };

    let categoria_id = match texto(fila, "categoria") {
        Some(nombre_cat) => match obtener_o_crear_categoria(tx, &nombre_cat) {
            Ok(id) => Some(id),
            Err(e) => return ResultadoFila::Error(e),
        },
        None => None,
    };
    let proveedor_id = match texto(fila, "proveedorPrincipal") {
        Some(razon) => match obtener_o_crear_proveedor(tx, &razon) {
            Ok(id) => Some(id),
            Err(e) => return ResultadoFila::Error(e),
        },
        None => None,
    };

    let existente: Option<i64> = match tx
        .query_row("SELECT id FROM productos WHERE sku = ?1", [&sku], |r| {
            r.get(0)
        })
        .optional()
    {
        Ok(v) => v,
        Err(e) => return ResultadoFila::Error(traducir_error_sql(&e)),
    };

    match (existente, modo) {
        (Some(_), ModoImport::Crear) => ResultadoFila::Omitido("Ya existe un producto con ese SKU.".into()),
        (None, ModoImport::Actualizar) => {
            ResultadoFila::Omitido("No existe un producto con ese SKU para actualizar.".into())
        }
        (Some(id), ModoImport::Actualizar) | (Some(id), ModoImport::CrearYActualizar) => {
            match tx.execute(
                "UPDATE productos SET codigo_barras=?1, nombre=?2, descripcion=?3, categoria_id=?4,
                    unidad_medida=?5, precio_costo=?6, precio_venta=?7, impuesto_pct=?8, stock_minimo=?9,
                    stock_maximo=?10, ubicacion=?11, proveedor_principal_id=?12, estado=?13,
                    actualizado_en = datetime('now')
                 WHERE id = ?14",
                params![
                    codigo_barras,
                    nombre,
                    descripcion,
                    categoria_id,
                    unidad_medida,
                    precio_costo,
                    precio_venta,
                    impuesto_pct,
                    stock_minimo,
                    stock_maximo,
                    ubicacion,
                    proveedor_id,
                    estado,
                    id
                ],
            ) {
                Ok(_) => ResultadoFila::Actualizado,
                Err(e) => ResultadoFila::Error(traducir_error_sql(&e)),
            }
        }
        (None, ModoImport::Crear) | (None, ModoImport::CrearYActualizar) => {
            match tx.execute(
                "INSERT INTO productos (sku, codigo_barras, nombre, descripcion, categoria_id, unidad_medida,
                    precio_costo, precio_venta, impuesto_pct, stock_minimo, stock_maximo, ubicacion,
                    proveedor_principal_id, estado)
                 VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14)",
                params![
                    sku,
                    codigo_barras,
                    nombre,
                    descripcion,
                    categoria_id,
                    unidad_medida,
                    precio_costo,
                    precio_venta,
                    impuesto_pct,
                    stock_minimo,
                    stock_maximo,
                    ubicacion,
                    proveedor_id,
                    estado
                ],
            ) {
                Ok(_) => ResultadoFila::Creado,
                Err(e) => ResultadoFila::Error(traducir_error_sql(&e)),
            }
        }
    }
}

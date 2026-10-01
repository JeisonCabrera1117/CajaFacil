pub mod ayudas;
pub mod categorias;
pub mod clientes;
pub mod lectura;
pub mod numeros;
pub mod plantillas;
pub mod productos;
pub mod proveedores;
pub mod stock_inicial;
pub mod ventas_historicas;

use std::collections::HashMap;

use rusqlite::Connection;

use crate::db::DbPool;
use crate::error::{AppError, AppResult};
use crate::models::importacion::{
    CampoImport, DeteccionArchivo, EntidadImportable, FilaRechazada, ModoImport, ResumenImportacion,
};

pub fn entidades_importables() -> Vec<EntidadImportable> {
    [
        ("productos", "Productos"),
        ("categorias", "Categorías"),
        ("proveedores", "Proveedores"),
        ("clientes", "Clientes"),
        ("stock_inicial", "Stock inicial"),
        ("ventas_historicas", "Ventas históricas"),
    ]
    .into_iter()
    .map(|(id, etiqueta)| EntidadImportable {
        id: id.into(),
        etiqueta: etiqueta.into(),
    })
    .collect()
}

pub enum ResultadoFila {
    Creado,
    Actualizado,
    Omitido(String),
    Error(String),
}

pub fn campos_de(entidad: &str) -> AppResult<Vec<CampoImport>> {
    Ok(match entidad {
        "productos" => productos::campos(),
        "categorias" => categorias::campos(),
        "proveedores" => proveedores::campos(),
        "clientes" => clientes::campos(),
        "stock_inicial" => stock_inicial::campos(),
        "ventas_historicas" => ventas_historicas::campos(),
        otro => {
            return Err(AppError::Validacion(format!(
                "Entidad de importación desconocida: {otro}"
            )))
        }
    })
}

fn procesar_fila(
    tx: &Connection,
    entidad: &str,
    fila: &HashMap<String, String>,
    modo: ModoImport,
) -> ResultadoFila {
    match entidad {
        "productos" => productos::procesar_fila(tx, fila, modo),
        "categorias" => categorias::procesar_fila(tx, fila, modo),
        "proveedores" => proveedores::procesar_fila(tx, fila, modo),
        "clientes" => clientes::procesar_fila(tx, fila, modo),
        "stock_inicial" => stock_inicial::procesar_fila(tx, fila),
        "ventas_historicas" => ventas_historicas::procesar_fila(tx, fila),
        otro => ResultadoFila::Error(format!("Entidad de importación desconocida: {otro}")),
    }
}

pub fn detectar(
    ruta: &str,
    hoja_solicitada: Option<&str>,
    separador_override: Option<char>,
) -> AppResult<DeteccionArchivo> {
    const FILAS_MUESTRA: usize = 5;

    if lectura::es_xlsx(ruta) {
        let hojas = lectura::listar_hojas(ruta)?;
        let hoja = hoja_solicitada
            .map(String::from)
            .or_else(|| hojas.first().cloned())
            .ok_or_else(|| AppError::Validacion("El archivo no tiene hojas.".into()))?;
        let leido = lectura::leer_xlsx(ruta, &hoja)?;
        Ok(DeteccionArchivo {
            tipo: "xlsx".into(),
            encoding: None,
            separador: None,
            hojas,
            hoja_seleccionada: Some(hoja),
            columnas: leido.encabezados,
            filas_muestra: leido.filas.iter().take(FILAS_MUESTRA).cloned().collect(),
            total_filas: leido.filas.len() as i64,
        })
    } else {
        let (encoding, separador, leido) = lectura::leer_csv(ruta, separador_override)?;
        Ok(DeteccionArchivo {
            tipo: "csv".into(),
            encoding: Some(encoding.to_string()),
            separador: Some(separador.to_string()),
            hojas: Vec::new(),
            hoja_seleccionada: None,
            columnas: leido.encabezados,
            filas_muestra: leido.filas.iter().take(FILAS_MUESTRA).cloned().collect(),
            total_filas: leido.filas.len() as i64,
        })
    }
}

fn leer_todo(
    ruta: &str,
    hoja: Option<&str>,
    separador: Option<char>,
) -> AppResult<lectura::ArchivoLeido> {
    if lectura::es_xlsx(ruta) {
        let hojas = lectura::listar_hojas(ruta)?;
        let hoja = hoja
            .map(String::from)
            .or_else(|| hojas.first().cloned())
            .ok_or_else(|| AppError::Validacion("El archivo no tiene hojas.".into()))?;
        lectura::leer_xlsx(ruta, &hoja)
    } else {
        lectura::leer_csv(ruta, separador).map(|(_, _, leido)| leido)
    }
}

/// Ejecuta la importación completa: lee el archivo, mapea cada fila a los
/// campos del sistema y procesa fila por fila dentro de una única
/// transacción. `confirmar = false` hace un "dry run" real (corre la misma
/// lógica de validación + escritura, pero revierte al final) — así el preview
/// usa exactamente la misma lógica que la ejecución real, sin necesidad de
/// mantener dos caminos de validación por separado.
#[allow(clippy::too_many_arguments)]
pub fn procesar(
    pool: &DbPool,
    ruta: &str,
    hoja: Option<&str>,
    separador: Option<char>,
    entidad: &str,
    mapeo: &HashMap<String, String>,
    modo: ModoImport,
    confirmar: bool,
    mut sobre_progreso: impl FnMut(i64, i64),
) -> AppResult<ResumenImportacion> {
    campos_de(entidad)?; // valida que la entidad exista

    let leido = leer_todo(ruta, hoja, separador)?;
    let total_filas = leido.filas.len() as i64;

    // mapeo: columna_archivo -> campo_sistema. Invertimos a índice_columna -> campo_sistema.
    let indices_por_campo: HashMap<&str, usize> = mapeo
        .iter()
        .filter_map(|(columna, campo)| {
            leido
                .encabezados
                .iter()
                .position(|c| c == columna)
                .map(|idx| (campo.as_str(), idx))
        })
        .collect();

    let mut conn = pool.get()?;
    let tx = conn.transaction()?;

    let mut resumen = ResumenImportacion {
        total_filas,
        ..Default::default()
    };
    let paso_progreso = (total_filas / 100).max(1);

    for (i, fila_cruda) in leido.filas.iter().enumerate() {
        let numero_fila = (i + 2) as i64; // fila 1 es el encabezado

        let mut fila: HashMap<String, String> = HashMap::new();
        for (campo, idx) in &indices_por_campo {
            if let Some(valor) = fila_cruda.get(*idx) {
                fila.insert((*campo).to_string(), valor.clone());
            }
        }

        match procesar_fila(&tx, entidad, &fila, modo) {
            ResultadoFila::Creado => resumen.creados += 1,
            ResultadoFila::Actualizado => resumen.actualizados += 1,
            ResultadoFila::Omitido(motivo) => {
                resumen.omitidos += 1;
                resumen.filas_rechazadas.push(FilaRechazada {
                    fila: numero_fila,
                    motivo,
                });
            }
            ResultadoFila::Error(motivo) => {
                resumen.con_error += 1;
                resumen.filas_rechazadas.push(FilaRechazada {
                    fila: numero_fila,
                    motivo,
                });
            }
        }

        if numero_fila % paso_progreso == 0 || i as i64 + 1 == total_filas {
            sobre_progreso(i as i64 + 1, total_filas);
        }
    }

    if confirmar {
        tx.commit()?;
    } else {
        tx.rollback()?;
    }

    Ok(resumen)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::importacion::ModoImport;
    use tempfile::tempdir;

    fn pool_de_prueba() -> DbPool {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.sqlite3");
        let pool = crate::db::init_pool(&path).unwrap();
        std::mem::forget(dir);
        pool
    }

    fn escribir_csv(contenido: &str) -> (tempfile::TempDir, String) {
        let dir = tempdir().unwrap();
        let ruta = dir.path().join("archivo.csv");
        std::fs::write(&ruta, contenido).unwrap();
        (dir, ruta.to_string_lossy().to_string())
    }

    fn escribir_xlsx(bytes: &[u8]) -> (tempfile::TempDir, String) {
        let dir = tempdir().unwrap();
        let ruta = dir.path().join("archivo.xlsx");
        std::fs::write(&ruta, bytes).unwrap();
        (dir, ruta.to_string_lossy().to_string())
    }

    fn mapeo_identidad(campos: &[&str]) -> HashMap<String, String> {
        campos
            .iter()
            .map(|c| (c.to_string(), c.to_string()))
            .collect()
    }

    #[test]
    fn importa_productos_y_crea_categoria_referenciada() {
        let pool = pool_de_prueba();
        let (_dir, ruta) = escribir_csv(
            "sku,nombre,categoria,precioCosto,precioVenta\n\
             SKU-1,Producto uno,Bebidas,1000,2000\n\
             SKU-2,Producto dos,Bebidas,1500,2500\n",
        );
        let mapeo = mapeo_identidad(&["sku", "nombre", "categoria", "precioCosto", "precioVenta"]);

        let resumen = procesar(
            &pool,
            &ruta,
            None,
            None,
            "productos",
            &mapeo,
            ModoImport::Crear,
            true,
            |_, _| {},
        )
        .unwrap();

        assert_eq!(resumen.creados, 2);
        assert_eq!(resumen.con_error, 0);
        assert_eq!(resumen.omitidos, 0);

        let conn = pool.get().unwrap();
        let total: i64 = conn
            .query_row("SELECT COUNT(*) FROM productos", [], |r| r.get(0))
            .unwrap();
        assert_eq!(total, 2);
        let categorias: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM categorias WHERE nombre = 'Bebidas'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(
            categorias, 1,
            "la categoría debe crearse una sola vez aunque la referencien dos filas"
        );
    }

    #[test]
    fn reimportar_en_modo_crear_omite_duplicados_por_sku() {
        let pool = pool_de_prueba();
        let (_dir, ruta) =
            escribir_csv("sku,nombre,precioCosto,precioVenta\nSKU-1,Producto uno,1000,2000\n");
        let mapeo = mapeo_identidad(&["sku", "nombre", "precioCosto", "precioVenta"]);

        procesar(
            &pool,
            &ruta,
            None,
            None,
            "productos",
            &mapeo,
            ModoImport::Crear,
            true,
            |_, _| {},
        )
        .unwrap();
        let segundo = procesar(
            &pool,
            &ruta,
            None,
            None,
            "productos",
            &mapeo,
            ModoImport::Crear,
            true,
            |_, _| {},
        )
        .unwrap();

        assert_eq!(segundo.creados, 0);
        assert_eq!(segundo.omitidos, 1);
        assert_eq!(segundo.filas_rechazadas[0].fila, 2);
    }

    #[test]
    fn modo_crear_y_actualizar_actualiza_precio_existente() {
        let pool = pool_de_prueba();
        let (_dir, ruta1) =
            escribir_csv("sku,nombre,precioCosto,precioVenta\nSKU-1,Producto uno,1000,2000\n");
        let mapeo = mapeo_identidad(&["sku", "nombre", "precioCosto", "precioVenta"]);
        procesar(
            &pool,
            &ruta1,
            None,
            None,
            "productos",
            &mapeo,
            ModoImport::Crear,
            true,
            |_, _| {},
        )
        .unwrap();

        let (_dir2, ruta2) =
            escribir_csv("sku,nombre,precioCosto,precioVenta\nSKU-1,Producto uno,1000,9999\n");
        let resumen = procesar(
            &pool,
            &ruta2,
            None,
            None,
            "productos",
            &mapeo,
            ModoImport::CrearYActualizar,
            true,
            |_, _| {},
        )
        .unwrap();

        assert_eq!(resumen.actualizados, 1);
        let conn = pool.get().unwrap();
        let precio: i64 = conn
            .query_row(
                "SELECT precio_venta FROM productos WHERE sku='SKU-1'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(precio, 999900);
    }

    #[test]
    fn preview_no_persiste_cambios() {
        let pool = pool_de_prueba();
        let (_dir, ruta) =
            escribir_csv("sku,nombre,precioCosto,precioVenta\nSKU-1,Producto uno,1000,2000\n");
        let mapeo = mapeo_identidad(&["sku", "nombre", "precioCosto", "precioVenta"]);

        let resumen = procesar(
            &pool,
            &ruta,
            None,
            None,
            "productos",
            &mapeo,
            ModoImport::Crear,
            false,
            |_, _| {},
        )
        .unwrap();
        assert_eq!(resumen.creados, 1);

        let conn = pool.get().unwrap();
        let total: i64 = conn
            .query_row("SELECT COUNT(*) FROM productos", [], |r| r.get(0))
            .unwrap();
        assert_eq!(
            total, 0,
            "el preview no debe dejar nada guardado en la base de datos"
        );
    }

    #[test]
    fn detecta_separador_punto_y_coma_y_reporta_fila_invalida() {
        let pool = pool_de_prueba();
        let (_dir, ruta) = escribir_csv(
            "sku;nombre;precioCosto;precioVenta\nSKU-1;Producto uno;1000;2000\nSKU-2;;1000;2000\n",
        );
        let mapeo = mapeo_identidad(&["sku", "nombre", "precioCosto", "precioVenta"]);

        let resumen = procesar(
            &pool,
            &ruta,
            None,
            None,
            "productos",
            &mapeo,
            ModoImport::Crear,
            true,
            |_, _| {},
        )
        .unwrap();

        assert_eq!(resumen.creados, 1);
        assert_eq!(resumen.con_error, 1);
        assert_eq!(resumen.filas_rechazadas[0].fila, 3);
    }

    #[test]
    fn detectar_reconoce_punto_y_coma_automaticamente() {
        let (_dir, ruta) = escribir_csv("sku;nombre\nSKU-1;Producto uno\n");
        let deteccion = detectar(&ruta, None, None).unwrap();
        assert_eq!(deteccion.separador.as_deref(), Some(";"));
        assert_eq!(deteccion.columnas, vec!["sku", "nombre"]);
        assert_eq!(deteccion.total_filas, 1);
    }

    #[test]
    fn stock_inicial_aplica_movimiento_de_entrada() {
        let pool = pool_de_prueba();
        {
            let conn = pool.get().unwrap();
            conn.execute(
                "INSERT INTO productos (sku, nombre, unidad_medida, precio_costo, precio_venta)
                 VALUES ('SKU-1', 'Producto uno', 'unidad', 0, 0)",
                [],
            )
            .unwrap();
        }
        let (_dir, ruta) = escribir_csv("sku,cantidad,costoUnitario\nSKU-1,50,1000\n");
        let mapeo = mapeo_identidad(&["sku", "cantidad", "costoUnitario"]);

        let resumen = procesar(
            &pool,
            &ruta,
            None,
            None,
            "stock_inicial",
            &mapeo,
            ModoImport::Crear,
            true,
            |_, _| {},
        )
        .unwrap();
        assert_eq!(resumen.creados, 1);

        let conn = pool.get().unwrap();
        let stock: i64 = conn
            .query_row(
                "SELECT stock_actual FROM productos WHERE sku='SKU-1'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(stock, 50);
    }

    #[test]
    fn ventas_historicas_crea_venta_completa_con_stock() {
        let pool = pool_de_prueba();
        {
            let conn = pool.get().unwrap();
            conn.execute(
                "INSERT INTO productos (sku, nombre, unidad_medida, precio_costo, precio_venta, stock_actual)
                 VALUES ('SKU-1', 'Producto uno', 'unidad', 1000, 2000, 100)",
                [],
            )
            .unwrap();
        }
        let (_dir, ruta) = escribir_csv(
            "fecha,sku,cantidad,precioUnitario,metodoPago,valorRecibido\n2026-01-15,SKU-1,3,2000,efectivo,10000\n",
        );
        let mapeo = mapeo_identidad(&[
            "fecha",
            "sku",
            "cantidad",
            "precioUnitario",
            "metodoPago",
            "valorRecibido",
        ]);

        let resumen = procesar(
            &pool,
            &ruta,
            None,
            None,
            "ventas_historicas",
            &mapeo,
            ModoImport::Crear,
            true,
            |_, _| {},
        )
        .unwrap();
        assert_eq!(resumen.creados, 1, "{:?}", resumen.filas_rechazadas);

        let conn = pool.get().unwrap();
        let stock: i64 = conn
            .query_row(
                "SELECT stock_actual FROM productos WHERE sku='SKU-1'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(stock, 97);
        let total: i64 = conn
            .query_row("SELECT total FROM ventas LIMIT 1", [], |r| r.get(0))
            .unwrap();
        assert_eq!(total, 600_000); // 3 * 2000 pesos, en centavos
    }

    #[test]
    fn lee_e_importa_un_xlsx_real() {
        // Usa la propia plantilla generada (rust_xlsxwriter) como archivo de entrada
        // y la importa (calamine) — ejercita todo el camino XLSX de punta a punta.
        let bytes = plantillas::generar_xlsx("productos").unwrap();
        let (_dir, ruta) = escribir_xlsx(&bytes);

        let deteccion = detectar(&ruta, None, None).unwrap();
        assert_eq!(deteccion.tipo, "xlsx");
        assert_eq!(deteccion.hojas, vec!["Sheet1"]);
        assert_eq!(deteccion.total_filas, 1);
        assert!(deteccion.columnas.contains(&"sku".to_string()));

        let pool = pool_de_prueba();
        let mapeo = mapeo_identidad(
            &campos_de("productos")
                .unwrap()
                .iter()
                .map(|c| c.id.as_str())
                .collect::<Vec<_>>(),
        );
        let resumen = procesar(
            &pool,
            &ruta,
            deteccion.hoja_seleccionada.as_deref(),
            None,
            "productos",
            &mapeo,
            ModoImport::Crear,
            true,
            |_, _| {},
        )
        .unwrap();

        assert_eq!(resumen.creados, 1, "{:?}", resumen.filas_rechazadas);
        let conn = pool.get().unwrap();
        let sku: String = conn
            .query_row("SELECT sku FROM productos LIMIT 1", [], |r| r.get(0))
            .unwrap();
        assert_eq!(sku, "PROD-001");
    }
}

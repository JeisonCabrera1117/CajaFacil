# CajaFácil

Aplicación de escritorio para Windows de administración de ventas e inventario,
para un solo usuario en un solo equipo. Construida con Tauri 2 (Rust), React y
TypeScript.

¿Querés levantarlo en tu máquina? Ver [`docs/EJECUTAR_LOCAL.md`](docs/EJECUTAR_LOCAL.md).
¿Cómo funciona por dentro? Ver [`docs/ARQUITECTURA.md`](docs/ARQUITECTURA.md).
¿Cómo está armado el diseño? Ver [`docs/DISENO.md`](docs/DISENO.md).
¿Qué falta para un instalador real? Ver [`docs/PENDIENTES.md`](docs/PENDIENTES.md).
¿Cómo se usa la aplicación? Ver [`docs/MANUAL_USUARIO.pdf`](docs/MANUAL_USUARIO.pdf).

## Estado del proyecto

Desarrollo por fases.

- **Fase 1** — Proyecto base, instalador, acceso con PIN y configuración. ✅ Completa.
- **Fase 2** — Inventario, proveedores y movimientos de stock. ✅ Completa.
- **Fase 3** — Ventas, clientes y caja. ✅ Completa.
- **Fase 4** — Comprobantes en PDF e imagen. ✅ Completa.
- **Fase 5** — Carga masiva (CSV/XLSX). ✅ Completa.
- **Fase 6** — Dashboard, gráficas y reportes. ✅ Completa.
- **Fase 7** — Respaldos, actualizaciones y pulido final. ✅ Completa.

### Detalle de la fase 2

**Backend Rust — completo, compila, `cargo clippy -D warnings` limpio, 13 tests unitarios en verde:**

- Modelos: `categoria`, `proveedor`, `producto`, `movimiento`, `compra`, `inventario_fisico`
  (`src-tauri/src/models/`).
- Servicios (`src-tauri/src/services/`):
  - `audit_service` — registra en `auditoria` cada crear/modificar/anular/eliminar.
  - `stock_service` — núcleo de movimientos de stock (entrada/salida/ajuste/devolución),
    costo promedio ponderado, kardex, bloqueo de stock negativo según configuración.
    Expone `aplicar_movimiento_tx` (para componer dentro de una transacción ya abierta,
    usado por compras e inventario físico) y `registrar_movimiento` (abre su propia
    transacción, usado por el ajuste manual desde el kardex). **Importante**: el pool de
    SQLite tiene `max_size = 1` (ver `db/mod.rs`); cualquier función que abra una conexión
    y luego llame a otra función que también hace `pool.get()` sin soltar la primera
    se queda esperando para siempre. Ya se pisó esa piedra dos veces (en `compra_service`
    y en `inventario_fisico_service::cerrar`) — la regla es: después de `tx.commit()`,
    hacer `drop(conn)` explícito antes de volver a pedir conexión al pool.
  - `categoria_service`, `proveedor_service` (incluye historial de compras y
    productos que suministra vía tabla `proveedor_productos`), `producto_service`
    (listar con filtro/paginación dinámica usando `rusqlite::types::Value`, CRUD;
    `stock_actual` **no** es editable directamente — solo cambia vía movimientos),
    `compra_service` (crear compra = transacción con inserts + movimiento de entrada
    por ítem; anular = revierte con movimiento de salida), `inventario_fisico_service`
    (iniciar → registrar conteos → cerrar aplica un ajuste por cada diferencia ≠ 0).
- Comandos Tauri registrados en `lib.rs`: `categoria_*`, `proveedor_*` (incluye
  `proveedor_historial_compras`, `proveedor_productos_list/asignar`), `producto_*`
  (incluye `producto_ajustar_stock`, `kardex_get`), `compra_*`, `inventario_fisico_*`.
- Sin migraciones nuevas: el esquema completo ya estaba en `migrations/0001_initial.sql`
  desde la fase 1.

**Frontend — completo, compilado y verificado:**

- `src/lib/api-inventario.ts` — tipos + wrappers `invoke` para todos los comandos de arriba.
- `src/features/productos/` (schema, `ProductoFormDialog`, `KardexDialog` con ajuste de
  stock inline, `ProductosPanel` con búsqueda/filtro stock-bajo/paginación/tabla),
  `src/features/categorias/CategoriasPanel.tsx`, `src/features/compras/` (`CompraFormDialog`,
  `ComprasPanel`), `src/features/inventario-fisico/InventarioFisicoPanel.tsx`,
  `src/features/proveedores/` (`ProveedorFormDialog`, `HistorialComprasDialog`,
  `ProductosQueSuministraDialog`, `ProveedoresPanel`).
- `src/pages/Inventario.tsx` (tabs: Productos / Categorías / Compras / Toma de inventario)
  y `src/pages/Proveedores.tsx`, ambos reemplazando el placeholder de la fase 1.
- Componentes shadcn agregados: `table`, `textarea`, `checkbox`, `alert-dialog`.
- `pnpm tsc --noEmit`, `pnpm eslint .` (0 errores, solo 4 warnings preexistentes de
  fase 1 sobre fast-refresh, inofensivos), `pnpm build` y `cargo build`/`cargo test`
  completos, todos en verde. `cargo fmt` y `pnpm run format` corridos.

**Bug real encontrado y corregido durante la verificación** (no lo detectan ni `tsc` ni
`eslint`, solo se ve al ejercitar el límite serde↔JSON): `ProductoFiltro`
(`src-tauri/src/models/producto.rs`) le faltaba `#[serde(default)]` a nivel de
contenedor. Sin eso, `producto_list` fallaba en tiempo de ejecución cada vez que el
frontend mandaba el filtro con solo algunas claves (p. ej. `CompraFormDialog` pidiendo
el catálogo completo solo con `{ pagina, porPagina }`) — serde exige que todas las
claves de un struct estén presentes salvo que el struct tenga `default`. Ya tiene test
de regresión (`models::producto::tests::filtro_acepta_json_con_claves_faltantes`).
Los demás structs `Deserialize` del proyecto se revisaron uno por uno: ninguno más
tiene este riesgo porque el frontend siempre manda todas sus claves (aunque sea `null`).

Probado también con un smoke test real: `cargo build` + levantar el binario bajo Xvfb
(sandbox sin GUI real) confirma que arranca, migra y no crashea con los ~30 comandos
nuevos registrados. **No se pudo hacer clic-a-clic en la UI** (no hay compositor gráfico
real disponible en este entorno) — probar el flujo completo (crear producto → ajustar
stock → comprar → anular → toma de inventario) queda pendiente de que lo hagas vos en
`pnpm tauri dev` sobre Windows o Linux con escritorio real.

Nada de esto está commiteado a git todavía (sigue como working tree sin commits).

### Detalle de la fase 3

**Backend Rust — completo, compila, `cargo clippy -D warnings` limpio, 25 tests unitarios en verde:**

- Modelos: `cliente`, `venta`, `caja`, `devolucion` (`src-tauri/src/models/`).
- Servicios (`src-tauri/src/services/`):
  - `cliente_service` — CRUD simple (sin borrar, como en el comando aprobado).
  - `caja_service` — abrir/cerrar sesión de caja con arqueo: `cerrar` calcula el
    monto de sistema sumando el efectivo neto (`valor_recibido - cambio`) de las
    ventas en efectivo de esa sesión y compara contra lo contado.
  - `venta_service` — el corazón del punto de venta:
    - `crear`: valida método de pago, calcula subtotal/descuentos (por línea +
      global)/impuestos/total, exige `valor_recibido >= total` si el pago es en
      efectivo (calcula el cambio), genera el número de comprobante con el
      prefijo y contador de `empresa_config` (e incrementa el contador en la
      misma transacción), y si `arqueo_activo` está prendido exige una caja
      abierta y la asocia a la venta. Cada línea baja stock vía
      `stock_service::aplicar_movimiento_tx` (tipo `salida`).
    - `anular`: revierte el stock (tipo `entrada`) y bloquea la anulación si la
      venta ya tiene devoluciones registradas (evitaría descuadrar el stock).
    - `listar`/`obtener`: igual patrón de filtro dinámico que `producto_service`
      (fecha desde/hasta, cliente, estado, método de pago). Cada línea trae
      `cantidadDevuelta` (subconsulta contra `devolucion_detalle`) para que el
      frontend sepa cuánto queda disponible para devolver.
  - `devolucion_service` — valida que la venta esté completada y que la
    cantidad a devolver no supere lo que queda disponible por línea (vendido
    menos ya devuelto); repone stock vía movimiento tipo `devolucion`. El
    valor devuelto es una aproximación (`subtotal_línea / cantidad`), documentado
    como tal en el código.
- Comandos Tauri: `cliente_*`, `caja_estado_actual`/`caja_abrir`/`caja_cerrar`,
  `venta_buscar_producto`/`venta_crear`/`venta_anular`/`venta_list`/`venta_get`,
  `devolucion_crear`.
- Sin migraciones nuevas (de nuevo, el esquema ya estaba completo desde la fase 1).
- `VentaFiltro` ya nació con `#[serde(default)]` a nivel de contenedor — se
  aprendió la lección de `ProductoFiltro` en la fase 2 y no se repitió el bug.

**Frontend — completo, compilado y verificado** (`tsc`, `eslint`, `pnpm build`,
`cargo build`, `cargo test`, `vitest`, todos en verde; sin errores nuevos esta
vez, ni siquiera de los que salieron en la fase 2 — el patrón `v ?? "..."` en los
`onValueChange` de los `Select` y evitar `z.coerce.number()` en formularios con
react-hook-form ya se aplicó desde el principio):

- `src/lib/api-ventas.ts` — tipos + wrappers para todos los comandos de arriba.
- `src/features/clientes/` (`ClienteFormDialog`, `ClientesPanel`) y
  `src/pages/Clientes.tsx`.
- `src/features/caja/CajaWidget.tsx` — banner de estado de caja (abrir/cerrar
  con arqueo), visible arriba de la página de Ventas.
- `src/features/ventas/`:
  - `Pos.tsx` — punto de venta: búsqueda de producto (por nombre/SKU/código de
    barras), carrito editable (cantidad, precio, descuento por línea), cliente
    opcional ("Cliente general" por defecto), descuento global, método de
    pago, cálculo de cambio en vivo, botón "Cobrar" deshabilitado hasta que
    el efectivo recibido alcance el total.
  - `HistorialVentasPanel.tsx` — filtros por fecha/estado/método de pago.
  - `VentaDetalleDialog.tsx` — detalle de la venta con botones de anular y
    devolver (solo cuando corresponde según estado y cantidades ya devueltas).
  - `DevolucionDialog.tsx` — selecciona cantidad a devolver por línea, respeta
    el máximo disponible.
- `src/pages/Ventas.tsx` reemplaza el placeholder de la fase 1 (caja + tabs
  Punto de venta / Historial).

Mismo smoke test que en fases anteriores: `cargo build` + binario bajo Xvfb
arranca y migra sin errores con los ~40 comandos ya registrados. Probar clic a
clic en la UI real sigue pendiente de que lo hagas vos (mismo motivo: sin
compositor gráfico en este entorno).

### Detalle de la fase 4

**Backend Rust — completo, compila, `cargo clippy -D warnings` limpio, 32 tests
unitarios en verde (incluye tests de integración reales que generan PDF/PNG de
verdad y verifican que el archivo quede en disco con contenido):**

- **Generación de PDF**: `printpdf` 0.7 (no `genpdf`, que dependía de una API
  más incierta) dibujando texto línea por línea con posicionamiento manual —
  mismo modelo de documento (`pdf::elementos::Elemento`) para los tres formatos,
  así la lógica de armar el recibo (montos, ítems, totales, leyenda) vive en un
  solo lugar (`pdf/elementos.rs`) y cada renderer solo la interpreta.
- **Generación de imagen**: `image` + `imageproc` + `ab_glyph`, con la fuente
  DejaVu Sans embebida en el binario (`src-tauri/assets/fonts/`, licencia
  Bitstream Vera/DejaVu, libre para redistribuir) — no depende de fuentes del
  sistema, así que funciona igual en cualquier Windows.
- **Logo de la empresa**: si `logoPath` apunta a una imagen válida, se dibuja
  centrado arriba del recibo en los tres formatos. Si la ruta está vacía, no
  existe o no es una imagen decodificable, se omite en silencio (no tiene
  sentido que falle una venta completa por un logo mal configurado).
- **Pool dinámico Rust/`image`**: no me generó bugs pero vale documentarlo —
  `printpdf` vendoriza internamente `image` 0.24.9 y `Image::from_dynamic_image`
  exige exactamente ese tipo, que no es compatible con el `image` 0.25 que usa
  el resto del código (el renderer de PNG, el copiado al portapapeles). Se
  agregó `image-para-printpdf` en `Cargo.toml` como alias de `image = "=0.24.9"`
  usado solo dentro de `pdf/render_pdf.rs` para cargar el logo antes de
  pasárselo a `printpdf`.
- **Bug real encontrado con un smoke test visual** (no lo detecta ningún test
  automatizado ni el compilador): las líneas separadoras del PDF quedaban
  encima del texto de arriba o cruzando los ascendentes del texto de abajo,
  porque `printpdf::use_text` posiciona por línea base, no por el borde
  superior del glifo, y el espacio que dejaba antes/después de cada línea
  (`0.8` del tamaño de fuente) no alcanzaba. Se corrigió dejando medio paso
  antes de la línea y un paso completo después. Se verificó generando un
  comprobante real, convirtiéndolo a PNG con `pdftoppm` (poppler) y mirando el
  resultado — antes y después del fix.
- `comprobantes_dir` (`%APPDATA%\CajaFacil\comprobantes\<año>\<mes>\...`) se
  agregó a `AppState`, calculado una sola vez al arrancar la app.
- Comandos: `comprobante_generar`, `comprobante_regenerar` (idénticos por
  debajo — generar una venta ya existente es exactamente lo mismo que
  "regenerar"), `comprobante_abrir` (via `tauri-plugin-opener`, ya estaba
  como dependencia desde la fase 1), `comprobante_copiar_imagen` (vía
  `arboard`, solo válido para el formato imagen).
- "Imprimir" no es un comando aparte: el botón "Abrir / Imprimir" del frontend
  abre el archivo con el lector de PDF/imágenes por defecto de Windows, donde
  el usuario imprime con Ctrl+P — así de simple, sin código nativo de
  impresión.

**Frontend — completo, compilado y verificado** (`tsc`, `eslint`, `pnpm build`,
`cargo build`, `cargo test`, `vitest`, todos en verde):

- `src/lib/api-comprobantes.ts` — tipos + wrappers.
- `src/features/comprobantes/ComprobanteDialog.tsx` — selector de formato
  (Carta / Térmico 80mm / Imagen), genera, y luego ofrece "Abrir / Imprimir" y,
  si el formato es imagen, "Copiar imagen".
- Se conectó en dos lugares: automáticamente después de cobrar una venta en el
  **POS** (`Pos.tsx`), y desde el botón "Comprobante" en el detalle de venta
  del **Historial** (`VentaDetalleDialog.tsx`, con `modo="regenerar"`) — ahí se
  puede regenerar el comprobante de cualquier venta anterior, tal como pide
  el requerimiento original.

### Detalle de la fase 5

**Backend Rust — completo, compila, `cargo clippy -D warnings` limpio, 48 tests
unitarios en verde:**

- Módulo nuevo `src-tauri/src/importacion/` (no `services/`, porque el
  "negocio" acá es leer/parsear/validar archivos, no solo persistencia):
  `lectura.rs` (CSV con `csv` + detección de codificación con `encoding_rs` +
  detección de separador; XLSX con `calamine`), `numeros.rs` (parseo tolerante
  de números en formato colombiano `1.234,56` o estándar `1,234.56`),
  `plantillas.rs` (genera CSV a mano y XLSX con `rust_xlsxwriter`), `ayudas.rs`
  (helpers compartidos: buscar-o-crear categoría/proveedor/cliente, traducir
  errores SQL), y un módulo por entidad (`productos`, `categorias`,
  `proveedores`, `clientes`, `stock_inicial`, `ventas_historicas`).
- **Diseño clave**: cada entidad expone una única función
  `procesar_fila(tx, fila, modo)` que valida **y** escribe en la misma
  pasada — no hay un camino de validación separado del de ejecución. El
  preview (`import_preview`) corre exactamente esa misma función fila por
  fila dentro de una transacción real y después la revierte
  (`tx.rollback()`); ejecutar (`import_ejecutar`) hace lo mismo pero con
  `tx.commit()`. Así la vista previa nunca puede decir "esto va a funcionar"
  y que la ejecución real se comporte distinto — es literalmente el mismo
  código. Esto también es lo que hace que una categoría referenciada por
  nombre en la fila 5 exista para la fila 8 si ambas están en el mismo
  archivo (se crea al vuelo dentro de la misma transacción).
- `stock_inicial` reutiliza `stock_service::aplicar_movimiento_tx` (ya
  expuesto para composición desde la fase 3); `ventas_historicas` arma la
  venta directamente (no pasa por `venta_service::crear`, que exige caja
  abierta si el arqueo está activo — no tiene sentido exigir eso para datos
  históricos) pero comparte la misma lógica de cálculo de totales/cambio y
  también actualiza stock vía `aplicar_movimiento_tx`.
- Progreso: `import_ejecutar` emite el evento `import-progreso` cada ~1% de
  filas procesadas (mínimo cada fila si el archivo es chico) para que el
  frontend muestre una barra sin bloquear la interfaz.
- Nuevo plugin `tauri-plugin-dialog` para el selector nativo de archivos
  (capability `dialog:default` agregada).
- **Bug real encontrado por los tests, no por el compilador**: al principio
  solo manejé el heurístico de miles/decimales para comas ("1.234,56") y me
  olvidé la rama espejo para cuando el archivo solo tiene puntos ("12.345")
  — caía a un `else` que devolvía el texto sin tocar, así que "1.234" se leía
  como 1.234 en lugar de 1234. Los tests de `numeros.rs` lo agarraron de
  inmediato.
- Se probó el camino XLSX de punta a punta con un truco simple: la plantilla
  XLSX que genera `rust_xlsxwriter` se usa como archivo de entrada para
  `calamine` en el mismo test — confirma que ambos crates son compatibles
  entre sí sin necesitar un archivo de ejemplo aparte.

**Frontend — completo, compilado y verificado** (`tsc`, `eslint`, `pnpm build`,
`cargo build`, `cargo test`, `vitest`, todos en verde):

- `src/lib/api-importacion.ts` — tipos + wrappers, más `descargarPlantilla()`
  (convierte el base64 que manda Rust en un `Blob` y dispara la descarga del
  navegador — no hace falta un diálogo de "guardar como" para esto).
- `src/features/carga-masiva/`:
  - `SeleccionArchivo.tsx` — elegir entidad, descargar plantilla, elegir
    archivo (diálogo nativo vía `@tauri-apps/plugin-dialog` **o** arrastrar y
    soltar vía el evento de ventana `onDragDropEvent` de Tauri), muestra tipo/
    codificación/separador detectados con overrides, y el selector de hoja
    para XLSX.
  - `mapeo.ts` — sugiere automáticamente qué columna del archivo corresponde
    a cada campo del sistema, comparando nombres normalizados (sin tildes,
    sin mayúsculas, sin símbolos).
  - `MapeoColumnas.tsx` — tabla para revisar/corregir esa sugerencia.
  - `ResumenPanel.tsx` — contadores + tabla de filas rechazadas (compartido
    entre la vista previa y el resultado final).
  - `CargaMasivaWizard.tsx` — orquesta los pasos, escucha `import-progreso`
    con `listen()` de `@tauri-apps/api/event` para la barra de progreso.
  - `HistorialImportacionesPanel.tsx` — con botón para abrir el archivo de
    filas rechazadas (reutiliza `apiComprobantes.abrir`, que ya era genérico).
- `src/pages/CargaMasiva.tsx` reemplaza el placeholder de la fase 1 (tabs
  Nueva importación / Historial).

Mismo smoke test que en fases anteriores. Una corrida del smoke test salió con
exit 0 casi inmediato en vez del exit 124 esperado (matando por timeout);
investigado por separado (proceso corriendo en segundo plano sin el wrapper
`timeout`) y confirmado que fue un problema transitorio de Xvfb (probablemente
un lock de una corrida anterior), no un bug de la app — el binario arranca y
queda estable normalmente.

### Detalle de la fase 6

**Backend Rust — completo, compila, `cargo clippy -D warnings` limpio, 61 tests
unitarios en verde:**

- `services/dashboard_service.rs` — `indicadores()` (ventas hoy/semana/mes,
  ticket promedio, número de ventas, utilidad bruta, productos con stock bajo,
  valor de inventario) y `graficas(desde, hasta)` (serie diaria, top/bottom 10
  productos, ventas por categoría, ventas por método de pago, y un
  comparativo contra el periodo inmediatamente anterior de igual duración,
  calculado con `chrono::NaiveDate`).
- **Limitación documentada, no bug**: la utilidad bruta (acá y en
  `reporte_service::utilidad`) usa el **costo actual** del producto, no el
  costo histórico al momento de cada venta — `venta_detalle` no guarda costo
  porque nunca se pensó para eso. Es la aproximación estándar cuando no se
  lleva costeo histórico por venta; está comentado en el código en los dos
  lugares donde aplica.
- `services/reporte_service.rs` — los 5 reportes pedidos (ventas con filtro
  por fecha/producto/categoría/cliente, utilidad por producto, inventario
  valorizado, rotación con detección de "sin movimiento" mirando _cualquier_
  movimiento de stock en el rango — no solo ventas, para no marcar como
  "sin rotación" un producto que solo tuvo un ajuste de conteo —, y compras
  por proveedor).
- Módulo nuevo `src-tauri/src/exportacion/` (paralelo a `pdf/`): exporta
  cualquier tabla genérica (columnas + filas de texto) a CSV, XLSX
  (`rust_xlsxwriter`) o PDF. El PDF usa un renderer de tabla propio en
  `exportacion/tabla_pdf.rs` (carta apaisada, columnas de ancho fijo, recorte
  de texto largo con "…" cuando no entra, paginación automática) — **no**
  reutiliza el renderer de recibos de la fase 4 (`pdf::render_pdf::Lienzo`),
  porque ese está pensado para una sola columna angosta tipo recibo, no para
  una grilla ancha de reporte.
- **Bug real encontrado con un smoke test visual** (otra vez ninguna prueba
  automatizada lo agarra): al exportar una tabla a PDF, las columnas casi no
  tenían separación entre sí — el margen de seguridad al truncar texto
  (2mm) no alcanzaba porque `ab_glyph` (usado para medir cuánto texto entra)
  y `printpdf` (usado para dibujarlo) no miden exactamente igual. Se
  confirmó generando un PDF de verdad, convirtiéndolo a PNG con `pdftoppm` y
  mirando el resultado — igual que el bug de separadores de la fase 4. Se
  subió el margen a 6mm y quedó bien. Ya tiene tests de regresión
  (`exportacion::tabla_pdf::tests`) que verifican que el texto recortado
  respeta el ancho máximo medido, aparte del chequeo visual puntual.
- Comandos: `dashboard_indicadores`, `dashboard_graficas`, `reporte_ventas`,
  `reporte_utilidad`, `reporte_inventario_valorizado`, `reporte_rotacion`,
  `reporte_compras_proveedor`, `reporte_exportar` (genérico, reutilizado por
  los 5 reportes desde el frontend).

**Frontend — completo, compilado y verificado** (`tsc`, `eslint`, `pnpm build`,
`cargo build`, `cargo test`, `vitest`, todos en verde):

- `src/lib/api-dashboard.ts`, `src/lib/api-reportes.ts` — tipos + wrappers.
- `src/lib/exportar-grafica.ts` — rasteriza el `<svg>` de una gráfica de
  Recharts a PNG (clona el nodo, lo serializa, lo dibuja en un `<canvas>` a
  2x y dispara la descarga) — cumple "gráficas exportables como imagen PNG"
  sin agregar ninguna dependencia nueva.
- `src/features/dashboard/`: `StatTiles.tsx` (8 indicadores), `AlertasStock.tsx`
  (tabla de productos bajo el mínimo, reutiliza el filtro `soloStockBajo` de
  `producto_list` que ya existía desde la fase 2), `GraficaVentas.tsx` (línea,
  con selector de rango de fechas), `GraficaProductos.tsx` (barras, más/menos
  vendidos), `GraficaDonas.tsx` (categoría y método de pago), `Comparativo.tsx`
  (tile de variación % contra el periodo anterior). `paleta.ts` fija el orden
  categórico de colores usando las variables `--chart-1`..`--chart-5` que ya
  existían en el tema de shadcn desde la fase 1 (nunca un color generado al
  vuelo; con más de 5 categorías se agrupa el resto en "Otros").
- `src/features/reportes/`: un panel por reporte con sus filtros propios +
  `ExportarReporteBotones.tsx` (compartido, 3 botones CSV/XLSX/PDF que llaman
  a `reporte_exportar` y reusan `descargarPlantilla()` de la fase 5 para el
  blob-download — la tabla que ve el usuario en pantalla es exactamente la
  que se exporta, ya formateada con `formatearMoneda`).
- `src/pages/Dashboard.tsx` y `src/pages/Reportes.tsx` reemplazan los
  placeholders de la fase 1.

No se pudo hacer una captura de pantalla real de la UI: este entorno no tiene
gestor de ventanas ni compositor, así que hasta con Xvfb la captura del
framebuffer sale en negro (se probó explícitamente para esta fase, dado que
son las primeras pantallas con gráficas). El proceso arranca y queda estable
igual que en las fases anteriores — falta que confirmes visualmente que las
gráficas se ven bien en `pnpm tauri dev` sobre un escritorio real.

### Detalle de la fase 7

**Backend Rust — completo, compila, `cargo clippy` limpio, 70 tests unitarios en verde (9 nuevos de esta fase):**

- `services/respaldo_service.rs` (nuevo):
  - `crear(pool, respaldos_dir, tipo)` — usa `VACUUM INTO` para generar un
    archivo `.sqlite3` consistente sin bloquear la conexión activa ni parar
    la app. Inserta la fila en la tabla `respaldos` (ya existía en el esquema
    desde la fase 1) y llama a `podar()`.
  - `podar(pool, limite)` — conserva solo los `limite` respaldos más recientes
    (20 en producción), borrando fila + archivo de los sobrantes.
  - `listar`, `eliminar`.
  - `solicitar_restauracion(pool, app_data_dir, id)` — **no restaura de
    inmediato**: mientras la app corre, el archivo de base de datos activo
    está abierto y no se puede reemplazar de forma segura. En vez de eso deja
    un archivo marcador (`restaurar.marcador`) con la ruta del respaldo
    elegido. El comando `respaldo_restaurar` (en `commands/respaldo_commands.rs`)
    llama a esto y después cierra la app (`app.exit(0)`). Al volver a abrir,
    `lib.rs::aplicar_restauracion_pendiente()` detecta el marcador **antes**
    de inicializar el pool, copia el respaldo sobre `cajafacil.sqlite3`, borra
    los archivos `-wal`/`-shm` viejos y el marcador, y recién ahí sigue el
    arranque normal.
- `services/audit_service.rs` ampliado: la función `registrar()` ya existía
  desde la fase 2 (se usa en 12 servicios distintos); se le agregó `listar()`
  con el mismo patrón de filtro dinámico (`rusqlite::types::Value` +
  `params_from_iter`) usado en `producto_service`/`reporte_service`, con
  filtro por entidad y rango de fechas, y paginación.
- Comandos nuevos: `respaldo_list`, `respaldo_crear`, `respaldo_eliminar`,
  `respaldo_restaurar`, `auditoria_list`.
- `AppState` ahora incluye `respaldos_dir` (`%APPDATA%\CajaFacil\respaldos\`).
- Plugins Tauri agregados: `tauri-plugin-updater` y `tauri-plugin-process`
  (este último solo para poder llamar `relaunch()` después de instalar una
  actualización). Permisos agregados en `capabilities/default.json`.
- Se generó un par de claves de firma para el updater con
  `pnpm tauri signer generate`. La clave pública quedó en `tauri.conf.json` →
  `plugins.updater.pubkey`; la privada **no se subió al repositorio** (está
  en `.gitignore` vía `*.key`) — ver la sección "Publicar actualizaciones"
  más abajo para cómo usarla en CI cuando se configure el hosting de releases.

**Frontend — completo, compilado y verificado:**

- `src/lib/api-respaldos.ts`, `src/lib/api-auditoria.ts` — wrappers `invoke`
  para los comandos de arriba.
- `src/features/config/RespaldosPanel.tsx` — botón "Crear respaldo ahora",
  tabla de respaldos con tamaño formateado, "Restaurar" (con `AlertDialog` de
  confirmación que avisa que la app se va a cerrar sola) y "Eliminar".
  "Abrir carpeta" reusa el comando genérico `comprobante_abrir` de la fase 4
  (abre cualquier ruta con el explorador de archivos del sistema) apuntando
  a la carpeta del respaldo más reciente.
- `src/features/config/AuditoriaPanel.tsx` — tabla paginada del historial de
  cambios con filtro por entidad y rango de fechas.
- `src/features/config/ActualizacionesPanel.tsx` — botón "Buscar
  actualizaciones" usando `@tauri-apps/plugin-updater` (`check()` /
  `downloadAndInstall()`) y `@tauri-apps/plugin-process` (`relaunch()`).
  Como todavía no hay servidor de actualizaciones real, hoy siempre muestra
  el mensaje de error esperado — ver "Publicar actualizaciones".
- `src/pages/Configuracion.tsx` reorganizado en pestañas (General / Respaldos
  / Actualizaciones / Historial de cambios) para no amontonar todo en una
  sola pantalla larga.
- Pulido final revisado en esta fase (ya existía desde la fase 1, se
  confirmó que sigue vigente): PIN con Argon2 (`services/auth_service.rs`),
  ícono de la app, `tauri.conf.json` con targets `msi`+`nsis`, hook de
  desinstalación NSIS que pregunta si conservar los datos
  (`installer-hooks.nsh`), workflow de GitHub Actions — este último se
  actualizó para leer los secretos de firma del updater si existen.

Igual que en la fase 6, no se pudo verificar visualmente la UI en este
entorno (sin gestor de ventanas). Se confirmó en cambio que el binario
compilado arranca y queda vivo bajo Xvfb con los dos plugins nuevos cargados
(sin panics de `setup()`), y que los 70 tests de backend + 5 de frontend
(Vitest) + `tsc`/`eslint`/`clippy`/`cargo fmt --check` pasan limpios.

## Requisitos

- [Node.js](https://nodejs.org/) 22 o superior y [pnpm](https://pnpm.io/) 10.
- [Rust](https://www.rust-lang.org/tools/install) (toolchain estable) vía `rustup`.
- En Linux, además, las librerías de desarrollo de WebKitGTK (solo necesario
  para desarrollar/probar en Linux; el instalador final es para Windows):
  ```
  sudo apt install libwebkit2gtk-4.1-dev libglib2.0-dev libgtk-3-dev \
    libsoup-3.0-dev build-essential libxdo-dev libssl-dev \
    libayatana-appindicator3-dev librsvg2-dev
  ```
- En Windows, [Visual Studio Build Tools](https://tauri.app/start/prerequisites/#windows)
  con el workload "Desktop development with C++" (Tauri lo requiere para
  compilar el binario nativo). WebView2 se instala automáticamente si falta.

## Ejecutar en desarrollo

```bash
pnpm install
pnpm tauri dev
```

Esto levanta Vite en modo desarrollo y abre la ventana de la aplicación.
Los datos (base de datos SQLite, logs, comprobantes, respaldos) se guardan en:

- Windows: `%APPDATA%\CajaFacil\`
- Linux (solo para desarrollo): `~/.local/share/com.daniel.cajafacil/`

## Pruebas

```bash
pnpm test              # Vitest (frontend)
cd src-tauri && cargo test   # pruebas unitarias de Rust
```

Linters:

```bash
pnpm lint               # eslint + prettier --check
cd src-tauri && cargo clippy --all-targets -- -D warnings
cd src-tauri && cargo fmt --check
```

## Generar el instalador de Windows

El instalador (.msi vía WiX y .exe vía NSIS) solo se puede compilar **desde
Windows** (WiX/NSIS son herramientas de Windows; no cross-compilan desde Linux).

```powershell
pnpm install
pnpm tauri build
```

Los instaladores quedan en:

- `src-tauri/target/release/bundle/msi/*.msi`
- `src-tauri/target/release/bundle/nsis/*.exe`

El desinstalador NSIS pregunta si se deben conservar los datos guardados en
`%APPDATA%\CajaFacil\` (ver `src-tauri/installer-hooks.nsh`). El instalador
MSI no borra esa carpeta bajo ningún caso (WiX no soporta ese diálogo
personalizado).

También existe un workflow de GitHub Actions (`.github/workflows/build-windows.yml`)
que compila el instalador en un runner de Windows. Por ahora se dispara
manualmente (`workflow_dispatch`) porque el proyecto no está conectado a un
repositorio remoto todavía.

## Publicar actualizaciones

El plugin de actualizaciones automáticas de Tauri (`tauri-plugin-updater`) está
conectado y la app tiene un botón "Buscar actualizaciones" en
**Configuración → Actualizaciones**. Lo que falta, porque todavía no se decidió
dónde alojar los releases (se respondió "solo local por ahora" durante el
desarrollo), es el **endpoint real**: `tauri.conf.json` → `plugins.updater.endpoints`
apunta a un dominio de ejemplo (`https://TODO-configurar-hosting-de-releases...`)
que nunca va a responder, así que hoy el botón siempre termina en "no se pudo
verificar" — es el comportamiento esperado, no un bug.

Para activarlo de verdad cuando se decida dónde alojar los releases (por
ejemplo GitHub Releases):

1. Reemplazar el/los endpoint(s) en `tauri.conf.json` por la URL real. El
   patrón `{{target}}/{{arch}}/{{current_version}}` ya está listo para el
   formato que espera el plugin.
2. La clave pública de firma ya está en `tauri.conf.json` →
   `plugins.updater.pubkey`. La clave privada correspondiente se generó con
   `pnpm tauri signer generate` y **no está en este repositorio** — Daniel la
   tiene guardada aparte (con contraseña vacía, ver nota de seguridad abajo).
   Para firmar builds en CI hay que cargarla como secretos de GitHub:
   `TAURI_SIGNING_PRIVATE_KEY` (contenido del archivo `.key`) y
   `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`. El workflow
   `.github/workflows/build-windows.yml` ya los lee si existen.
3. Publicar el `latest.json` que genera `tauri build` junto con los
   instaladores en el endpoint elegido.

**Nota de seguridad**: la clave se generó con contraseña vacía para no
bloquear el desarrollo sin CI todavía configurado. Antes de publicar
actualizaciones reales conviene regenerarla con `pnpm tauri signer generate
-w ruta/clave.key` y una contraseña, actualizando el secreto
`TAURI_SIGNING_PRIVATE_KEY_PASSWORD` en GitHub y el `pubkey` en
`tauri.conf.json`.

También está preparada la firma de código del instalador de Windows (aparte de
la firma de actualizaciones): se puede configurar un certificado en
`tauri.conf.json` → `bundle.windows.certificateThumbprint` (o variables de
entorno `TAURI_SIGNING_PRIVATE_KEY`/`...WINDOWS_...`) cuando se tenga uno, sin
cambios de arquitectura adicionales.

## Estructura de carpetas

```
src-tauri/           Backend Rust (Tauri)
  src/
    commands/        Comandos expuestos al frontend (capa delgada)
    services/        Lógica de negocio
    db/               Conexión SQLite + migraciones embebidas
    models/           Structs compartidos (serde)
  migrations/         SQL versionado (rusqlite_migration)
  icons/              Íconos de la app (todas las resoluciones)
  installer-hooks.nsh Hook del instalador NSIS (preguntar por datos al desinstalar)
src/                  Frontend React + TypeScript
  pages/              Una página por módulo
  layout/             Shell de la aplicación (barra lateral)
  features/           Lógica por dominio (auth, config, ...)
  components/ui/      Componentes shadcn/ui
  lib/                Wrappers de invoke, formato de moneda/fecha, etc.
templates/            Plantillas de carga masiva (se agregan en fase 5)
.github/workflows/    CI para compilar el instalador
```

## Cómo probar la fase 1

1. `pnpm install && pnpm tauri dev`.
2. La app abre directo en el Dashboard (sin PIN configurado todavía).
3. Ir a **Configuración**, completar los datos de la empresa y guardar.
4. En la sección "Acceso con PIN", activar un PIN (4 a 8 dígitos). La app
   recarga; al reabrir pedirá el PIN.
5. Verificar que un PIN incorrecto es rechazado y uno correcto desbloquea.
6. Desactivar el PIN desde Configuración (pide el PIN actual) y confirmar que
   ya no se solicita al reabrir.
7. Revisar que se creó `%APPDATA%\CajaFacil\cajafacil.sqlite3` (o el
   equivalente en Linux) y la carpeta `logs\`.

## Cómo probar la fase 2

1. `pnpm tauri dev` (o seguir desde la fase 1 si ya está corriendo).
2. En **Inventario → Categorías**: crear una o dos categorías (probar también
   con categoría padre).
3. En **Proveedores**: crear un proveedor. Verificar que "Compras" muestra
   vacío y que "Productos" permite abrir el diálogo (aunque no haya productos
   todavía).
4. En **Inventario → Productos**: crear un producto (SKU único, precios,
   categoría y proveedor recién creados). Confirmar que el stock arranca en 0.
5. Abrir el stock del producto (clic en el número) → **Ajustar stock** → tipo
   "Entrada", cantidad y costo unitario → registrar. Verificar que el stock y
   el kardex se actualizan.
6. Probar un ajuste con tipo "Ajuste" sin motivo — debe rechazarlo — y luego
   con motivo, debe funcionar (puede ser negativo para restar).
7. En **Inventario → Compras**: registrar una compra con 1-2 ítems del
   producto creado. Verificar que el stock sube y que aparece en el kardex
   con referencia "compra". Anular la compra y verificar que el stock vuelve
   a bajar y que no se puede anular dos veces.
8. En **Inventario → Toma de inventario**: iniciar una toma, contar el
   producto con un valor distinto al stock actual, cerrar la toma y verificar
   que el stock se ajustó y que el kardex muestra el movimiento de tipo
   "ajuste" con el motivo automático.
9. Probar eliminar el producto: si ya tiene movimientos (los pasos anteriores
   se los generan), debe rechazar el borrado con un mensaje claro sugiriendo
   desactivarlo en su lugar.
10. Probar filtro "Stock bajo" en Productos con un producto cuyo stock quedó
    por debajo del mínimo configurado.

## Cómo probar la fase 3

1. Seguir teniendo al menos un producto con stock (de la fase 2) y, en
   **Configuración**, dejar "Arqueo de caja diario" desactivado para el primer
   intento (así no hace falta abrir caja para vender).
2. En **Clientes**: crear un cliente. Confirmar que el buscador filtra por
   nombre/documento/teléfono.
3. En **Ventas → Punto de venta**: buscar el producto creado, agregarlo al
   carrito, ajustar cantidad y un descuento por línea. Elegir "Cliente
   general" o el cliente recién creado. Método de pago "Efectivo": probar que
   "Cobrar" queda deshabilitado si el valor recibido es menor al total, y que
   el cambio se calcula bien al poner un valor mayor.
4. Cobrar la venta. Verificar en **Inventario → Productos** que el stock bajó
   y que el kardex muestra un movimiento "salida" con referencia "venta".
5. En **Ventas → Historial**: ver la venta recién creada, abrir su detalle,
   confirmar los totales.
6. Activar "Arqueo de caja diario" en Configuración, volver a Ventas: debe
   aparecer "Caja cerrada" y el botón "Cobrar" debe rechazar la venta hasta
   abrir la caja. Abrir caja con un monto de apertura, vender algo en
   efectivo, y cerrar la caja: verificar que el monto de sistema y la
   diferencia calculan bien (sistema = apertura + efectivo neto de las ventas
   de esa sesión).
7. Desde el detalle de una venta completada: registrar una devolución parcial
   de un producto. Verificar que el stock sube de nuevo y que el detalle
   muestra la cantidad ya devuelta. Confirmar que no se puede devolver más de
   lo que queda disponible.
8. Anular una venta sin devoluciones: el stock debe volver al valor anterior.
   Confirmar que una venta con devoluciones ya registradas no se puede anular
   (debe mostrar un mensaje claro).
9. Probar el filtro de fecha/estado/método de pago en el historial.

## Cómo probar la fase 4

1. Completar los datos de la empresa en **Configuración** (nombre, NIT,
   dirección, teléfono) para verlos reflejados en el comprobante. Opcional:
   poner una ruta de logo válida (una imagen PNG/JPG que exista en el disco).
2. Registrar una venta en el POS. Al cobrar debería abrirse automáticamente el
   diálogo "Comprobante de venta".
3. Probar los tres formatos uno por uno: generar "PDF carta", luego "Abrir /
   Imprimir" y confirmar que abre con el lector de PDF por defecto. Repetir
   con "PDF térmico 80mm" (la página debe ser angosta, sin espacio en blanco
   de sobra). Repetir con "Imagen (PNG)".
4. Con el formato imagen generado, probar "Copiar imagen" y pegar (Ctrl+V) en
   cualquier programa que acepte imágenes del portapapeles (un chat, un
   editor de imágenes) para confirmar que copió algo válido.
5. Verificar que el archivo quedó en
   `%APPDATA%\CajaFacil\comprobantes\<año>\<mes>\<numeroComprobante>-<tipo>.<ext>`.
6. Ir al historial de ventas, abrir el detalle de esa misma venta, y usar el
   botón "Comprobante" para regenerarlo — confirmar que crea un archivo nuevo
   (no falla ni pisa el anterior).
7. Si se configuró un logo: confirmar que aparece centrado arriba en los tres
   formatos. Poner una ruta de logo inválida (que no exista) y confirmar que
   igual genera el comprobante, simplemente sin logo.

## Cómo probar la fase 5

1. Ir a **Carga masiva → Nueva importación**. Elegir la entidad "Productos" y
   descargar la plantilla CSV.
2. Abrir la plantilla descargada, agregar 2-3 filas de productos con SKUs que
   no existan todavía (dejar la fila de ejemplo o borrarla). Guardar.
3. Arrastrar el archivo a la zona de "soltar" (o usar "Elegir archivo…") —
   confirmar que detecta tipo CSV, codificación UTF-8 y separador coma, y que
   la tabla de mapeo ya viene con las columnas sugeridas automáticamente.
4. Modo "Solo crear" → "Vista previa": confirmar que el resumen muestra los
   creados esperados y 0 errores.
5. "Ejecutar importación": ver la barra de progreso (aunque con pocas filas
   pase muy rápido) y el resumen final. Verificar en **Inventario → Productos**
   que aparecen los productos nuevos.
6. Repetir la importación del mismo archivo en modo "Solo crear": debe omitir
   todas las filas (SKU duplicado). Cambiar a "Crear y actualizar" y modificar
   un precio en el archivo: debe actualizar ese producto.
7. Probar con un archivo con errores a propósito (una fila sin nombre, un
   precio no numérico): confirmar que la vista previa cuenta "con error" y
   lista el motivo por fila; en el historial, verificar que aparece el botón
   "Ver rechazados" y que abre un CSV legible con esas filas.
8. Probar con un archivo separado por punto y coma (guardar el CSV desde Excel
   en español suele generar esto): confirmar que lo detecta solo, o cambiarlo
   manualmente con el selector de separador.
9. Repetir el flujo con la entidad "Stock inicial" sobre un producto ya
   existente: confirmar que su stock sube y que aparece un movimiento en su
   kardex. Probar también "Ventas históricas" con una fecha pasada y confirmar
   que la venta aparece en el historial de ventas con esa fecha.
10. Probar con un archivo XLSX (la plantilla XLSX descargada sirve) para
    confirmar que también funciona, incluyendo el selector de hoja si el
    archivo tiene más de una.

## Cómo probar la fase 6

1. Tener datos de fases anteriores: al menos un producto con stock bajo (para
   ver la alerta), varias ventas en distintos días del mes actual, alguna
   compra, y algún producto sin ninguna venta reciente.
2. Ir a **Dashboard**: confirmar que los 8 indicadores muestran números
   razonables (compararlos a ojo contra el historial de ventas). Si hay algún
   producto con stock bajo, debe aparecer la tarjeta "Alertas de stock mínimo"
   con esa tabla; si no hay ninguno, la tarjeta no debe mostrarse.
3. En la gráfica "Ventas por día": cambiar el rango de fechas y confirmar que
   la línea y el resto de gráficas de abajo (más/menos vendidos, categoría,
   método de pago, comparativo) se actualizan juntas. Probar un rango sin
   ventas y confirmar que cada gráfica muestra su mensaje de "sin datos" en
   vez de romperse.
4. Probar "Exportar PNG" en al menos una gráfica: confirmar que descarga un
   PNG legible con el contenido de la gráfica.
5. Ir a **Reportes → Ventas**: probar los filtros (fecha, producto, categoría,
   cliente) por separado y combinados. Exportar a CSV, XLSX y PDF, y abrir
   cada archivo para confirmar que el contenido coincide con la tabla en
   pantalla (los montos deben verse formateados igual, ej. "$ 15.000").
6. Repetir una exportación rápida en **Utilidad**, **Inventario valorizado**,
   **Rotación** y **Compras por proveedor** — alcanza con probar un formato
   por reporte, ya que los tres comparten el mismo exportador genérico.
7. En **Rotación**, confirmar que un producto sin ventas ni movimientos en el
   rango elegido aparece marcado "Sin movimiento", y que uno con ventas no.
8. Si un reporte tiene muchas filas (o generalo importando un CSV grande de
   la fase 5), exportarlo a PDF y confirmar que pagina correctamente en vez de
   cortar contenido.

## Cómo probar la fase 7

1. Ir a **Configuración → Respaldos** y hacer clic en "Crear respaldo ahora":
   debe aparecer en la tabla con fecha, tipo "manual" y un tamaño razonable
   (unos KB con pocos datos de prueba). Repetirlo un par de veces más.
2. Clic en "Abrir carpeta": debe abrir `%APPDATA%\CajaFacil\respaldos\` en el
   explorador de archivos, con los `.sqlite3` generados.
3. Crear un producto nuevo (o cualquier cambio fácil de notar), luego ir a
   **Respaldos** y "Restaurar" uno de los respaldos creados **antes** de ese
   cambio. Confirmar el diálogo de advertencia: la app debe cerrarse sola.
   Volver a abrirla manualmente y confirmar que el cambio hecho después del
   respaldo ya no está (la base de datos volvió al estado de ese respaldo).
4. Eliminar un respaldo desde la tabla y confirmar que desaparece de la lista
   y del archivo en disco.
5. Crear más de 20 respaldos seguidos (o bajar `LIMITE_RESPALDOS` temporalmente
   para probarlo rápido) y confirmar que solo se conservan los 20 más
   recientes.
6. Ir a **Configuración → Historial de cambios**: confirmar que aparecen las
   acciones hechas durante las pruebas de fases anteriores (crear producto,
   crear venta, etc.). Probar el filtro por entidad y por rango de fechas, y
   la paginación con "Anterior"/"Siguiente" si hay muchos registros.
7. Ir a **Configuración → Actualizaciones**: clic en "Buscar actualizaciones".
   Como el endpoint todavía es un placeholder (ver "Publicar actualizaciones"
   en este README), es normal y esperado que salga el toast de error "No se
   pudo verificar actualizaciones…" — confirma que el botón no rompe nada, no
   que encuentre una actualización real.
8. Generar el instalador (`pnpm tauri build`) y confirmar que sigue
   funcionando igual que antes: instala, aparece en Agregar o quitar
   programas, y al desinstalar pregunta si conservar los datos.

## Proyecto completo

Con la fase 7 se cerraron las siete fases planeadas. Antes de usar la
aplicación en un negocio real, conviene:

- Probar visualmente la UI completa en un Windows real (este entorno de
  desarrollo no tiene forma de renderizar ventanas para verificarlo a ojo).
- Decidir dónde alojar las actualizaciones y completar los tres pasos de
  "Publicar actualizaciones" cuando se quiera activar el auto-update de
  verdad.
- Revisar `src-tauri/tauri.conf.json` (nombre, versión, ícono) antes del
  primer release público.

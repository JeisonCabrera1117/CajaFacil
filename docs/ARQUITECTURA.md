# Arquitectura

Qué tiene el proyecto y cómo está armado por dentro. Para instrucciones de
"cómo lo corro" o "cómo se ve", ver `EJECUTAR_LOCAL.md` y `DISENO.md` en esta
misma carpeta; para lo que falta, `PENDIENTES.md`.

## Qué es

Aplicación de escritorio para Windows (un solo usuario, un solo equipo) para
gestionar ventas e inventario de un negocio pequeño: catálogo de productos,
compras a proveedores, ventas tipo POS, comprobantes en PDF/imagen, carga
masiva desde Excel/CSV, dashboard con indicadores y gráficas, reportes
exportables, respaldos de la base de datos y actualizaciones automáticas.

## Stack

| Capa          | Tecnología                                                                       | Por qué                                                            |
| ------------- | -------------------------------------------------------------------------------- | ------------------------------------------------------------------ |
| Empaquetado   | Tauri 2                                                                          | binario nativo liviano (webview del sistema, no Chromium embebido) |
| Backend       | Rust                                                                             | toda la lógica de negocio vive acá, no en el frontend              |
| Base de datos | SQLite (`rusqlite` + `r2d2`)                                                     | un solo archivo, sin servidor que instalar                         |
| Frontend      | React 19 + TypeScript + Vite                                                     | UI                                                                 |
| Estilos       | Tailwind CSS v4 + shadcn/ui ("base-nova", sobre `@base-ui/react`)                | ver `DISENO.md`                                                    |
| Estado remoto | TanStack Query                                                                   | cachea y sincroniza las respuestas de los comandos Tauri           |
| Gráficas      | Recharts                                                                         | dashboard y reportes                                               |
| Documentos    | `printpdf`, `image`/`imageproc`/`ab_glyph`, `rust_xlsxwriter`, `calamine`, `csv` | comprobantes, exportación de reportes, carga masiva                |

## El viaje de una acción: UI → Rust → SQLite

No hay API HTTP ni REST: el frontend llama funciones de Rust directamente
vía el puente IPC de Tauri (`invoke("nombre_comando", args)`), y Tauri
serializa/deserializa con `serde`. El flujo es siempre:

```
Componente React
  → src/lib/api-*.ts        (invoke() tipado, un archivo por dominio)
  → src-tauri/src/commands/*  (capa delgada: recibe el State<AppState>, llama al servicio)
  → src-tauri/src/services/*  (TODA la lógica de negocio y las queries SQL)
  → src-tauri/src/db/          (pool de conexión SQLite)
```

**Las reglas de negocio nunca viven en un comando ni en el frontend.** Un
comando (`#[tauri::command]`) es una línea o dos: desempaqueta el estado y
delega al servicio. Si hace falta validar algo, calcular algo o tocar más de
una tabla, eso va en `services/`.

- 19 archivos en `services/`, 18 en `commands/`, **72 comandos** Tauri
  registrados en `lib.rs`.
- Tipos compartidos (lo que cruza el puente IPC) en `models/` (Rust) y
  reflejados a mano como interfaces TypeScript en cada `lib/api-*.ts` — misma
  forma, `#[serde(rename_all = "camelCase")]` de un lado, `camelCase` nativo
  del otro.

## Base de datos

Un archivo SQLite en `%APPDATA%\CajaFacil\cajafacil.sqlite3` (nunca dentro de
la carpeta de instalación). 20 tablas, todas en una sola migración
(`src-tauri/migrations/0001_initial.sql`) aplicada con `rusqlite_migration` —
nunca hizo falta una segunda porque el esquema completo se pensó antes de
empezar a programar.

Grupos de tablas:

- **Catálogo**: `productos`, `categorias`, `proveedores`, `proveedor_productos`.
- **Compras e inventario**: `compras`, `compra_detalle`, `movimientos_stock`
  (kardex — toda entrada/salida/ajuste/devolución pasa por acá),
  `inventario_fisico`, `inventario_fisico_detalle`.
- **Ventas**: `clientes`, `caja_sesiones`, `ventas`, `venta_detalle`,
  `devoluciones`, `devolucion_detalle`.
- **Documentos y trazabilidad**: `comprobantes`, `importaciones`,
  `auditoria`, `respaldos`.
- **Config**: `empresa_config` (fila única, `id = 1`).

**Detalle importante de infraestructura**: el pool de conexiones
(`db/mod.rs`) se crea con `max_size(1)` a propósito — SQLite serializa las
escrituras de todos modos, así que una sola conexión evita el error
"database is locked" al abrir varias conexiones en frío. La consecuencia es
que **nunca hay que pedir una segunda conexión al pool mientras la primera
sigue viva** (ni directa ni indirectamente, llamando a otra función que
también hace `pool.get()`): se traba. El patrón en todo el código es `drop(conn)`
explícito antes de cualquier llamada subsiguiente que también use el pool.

## Decisiones que se repiten en todo el código

- **Dinero = enteros (centavos), nunca `f64`.** De la base de datos al
  frontend y de vuelta. `formatearMoneda`/`formatear_moneda` (duplicado a
  propósito en TS y Rust) convierte a texto en el borde de la UI/PDF, nunca
  antes. COP se muestra sin decimales; otras monedas, con 2.
- **Errores**: `AppError` (Rust) solo deja cruzar un mensaje en español hacia
  el frontend (`mensaje_usuario()`); el detalle técnico se va al log
  (`tracing`, archivo diario en `%APPDATA%\CajaFacil\logs\`). El frontend
  nunca necesita traducir ni interpretar el error, solo mostrarlo.
- **Auditoría automática**: cada `crear`/`modificar`/`anular`/`eliminar` en
  los servicios de negocio llama a `audit_service::registrar(...)` — no es
  opt-in, está cableado adentro de la función del servicio. Se revisa en
  Configuración → Historial de cambios.
- **Fechas: SQLite guarda en UTC, todo lo que agrupa "por día" debe pedir
  `'localtime'` explícito.** `datetime('now')` sin modificador es UTC; para
  un usuario en Colombia (UTC-5) comparar eso contra una fecha que el Rust
  calculó con `chrono::Local` corre el corte del día 5 horas y rompe
  "ventas de hoy" cada noche a partir de las 7pm. Ver el comentario al tope
  de `services/dashboard_service.rs`.
- **Filtros dinámicos**: cuando un listado tiene 3+ filtros opcionales
  (productos, ventas, reportes, auditoría), se arma el `WHERE` a mano con
  `Vec<String>` + `Vec<rusqlite::types::Value>` y se ejecuta con
  `params_from_iter` — ver cualquier `*_service.rs` con una función
  `listar(pool, filtro)`.
- **Transacciones reales, no solo "varios INSERT seguidos"**: crear una
  venta, una compra, cerrar un inventario físico o ejecutar una carga masiva
  abren una transacción SQLite de verdad y hacen rollback si algo falla a la
  mitad — nunca queda un movimiento de stock sin su venta, o una fila
  importada sin las demás de su misma corrida.

## Los 7 módulos (fases del desarrollo)

Cada uno tiene su detalle técnico y su checklist de prueba manual en el
README, bajo "Detalle de la fase N" / "Cómo probar la fase N".

1. **Base**: proyecto Tauri, PIN con Argon2, configuración de empresa, instalador.
2. **Inventario**: productos, categorías, proveedores, compras, kardex, toma de inventario física.
3. **Ventas**: POS, clientes, caja (apertura/cierre), historial, anulación/devoluciones.
4. **Comprobantes**: PDF carta, PDF térmico 80mm y PNG, generados desde un mismo modelo de elementos (`pdf/elementos.rs`) compartido entre los tres renderizadores.
5. **Carga masiva**: CSV/XLSX con detección de encoding/separador, mapeo de columnas, preview que corre el mismo código que la ejecución real (dentro de una transacción con rollback si se cancela), exportación de rechazados.
6. **Dashboard y reportes**: indicadores, gráficas (Recharts, paleta categórica fija — ver `features/dashboard/paleta.ts`), 5 reportes exportables a CSV/XLSX/PDF.
7. **Respaldos, actualizaciones y pulido**: `VACUUM INTO` para respaldos consistentes, restauración vía archivo marcador + reinicio (la conexión activa no se puede pisar en caliente), `tauri-plugin-updater` cableado (endpoint todavía placeholder, ver `PENDIENTES.md`).

## Frontend: cómo están organizadas las pantallas

```
src/
  pages/            una página por módulo (Dashboard.tsx, Inventario.tsx, ...)
  layout/AppShell.tsx   sidebar + layout general de la app
  features/<dominio>/   componentes específicos de cada módulo (paneles, diálogos, formularios)
  components/ui/        shadcn/ui (Button, Card, Badge, Table, ...) — no se tocan a mano, se extienden por variante (cva)
  lib/
    api-*.ts        un wrapper invoke() tipado por dominio, espejo 1:1 de los comandos Rust
    format.ts        formatearMoneda/formatearFecha/formatearTamano, usados en toda la UI
    tauri.ts          wrapper de configuración de empresa
```

Patrón de datos: TanStack Query para todo lo que viene del backend
(`useQuery`/`useMutation` + `invalidateQueries` tras cada mutación), React
Hook Form + Zod para formularios. No hay estado global tipo Redux — cada
página pide lo que necesita.

## Qué corre dónde (seguridad y entorno)

- Node/Vite y el frontend solo existen en tiempo de desarrollo/build; lo que
  se instala en la máquina del usuario es el binario Rust + los archivos
  estáticos ya compilados (`dist/`).
- No hay red: no hay API externa, no hay telemetría. La única conexión de
  red prevista es el chequeo de actualizaciones (`tauri-plugin-updater`),
  hoy apuntando a un endpoint placeholder.
- PIN opcional con Argon2 (`services/auth_service.rs`) — protege el acceso a
  la app, no cifra la base de datos en disco.

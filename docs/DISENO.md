# Diseño de CajaFácil

Referencia rápida del sistema visual, para mantenerlo consistente en pantallas
nuevas. Inspirado en el lenguaje visual de monday.com: azul de acento, fondo
gris claro con tarjetas blancas, y pills de estado bien saturadas en vez de
badges discretos.

Todo vive como variables CSS en `src/index.css` (una vez en `:root` para tema
claro, otra vez en `.dark` para oscuro) y de ahí lo heredan automáticamente
los componentes de `src/components/ui/` (Button, Badge, Card, Table, Input...).
**Nunca hardcodear un color en un componente** — si hace falta un color nuevo,
se agrega como variable en `index.css` y se referencia con `var(--...)` o la
clase de Tailwind correspondiente (`bg-primary`, `text-warning`, etc.).

## Color

| Token                | Claro                        | Oscuro                      | Uso                                                              |
| -------------------- | ---------------------------- | --------------------------- | ---------------------------------------------------------------- |
| `--primary`          | `#0073ea`                    | `#3d94f5`                   | Acento principal: botones, links, ítem de menú activo            |
| `--success`          | `#00c875`                    | `#2ecf82`                   | Estado positivo: venta completada, compra registrada, "creado"   |
| `--warning`          | `#d98216`                    | `#fdab3d`                   | Alerta / precaución: stock bajo, inventario abierto, con errores |
| `--destructive`      | `#e2445c`                    | `#ff6b81`                   | Error / irreversible: agotado, anulado, eliminado                |
| `--background`       | gris `oklch(0.98 0.004 286)` | navy oscuro                 | Fondo de la app                                                  |
| `--card`             | blanco                       | navy más claro que el fondo | Tarjetas, tablas, diálogos                                       |
| `--muted-foreground` | gris medio                   | gris claro                  | Texto secundario                                                 |

`--chart-1` a `--chart-5` son una paleta **categórica** aparte (azul, morado,
verde, ámbar, gris) para gráficas — no reutilizan `--success`/`--warning`/
`--destructive` a propósito, para que un segmento de gráfica en rojo o verde
nunca se confunda con un estado de error o éxito. Ver
`src/features/dashboard/paleta.ts`.

`--radius` es `0.75rem`; todo lo demás (`--radius-sm/md/lg/xl...`) se deriva
de ese único valor en el bloque `@theme inline`.

## Badges: cuándo usar cada variante

`src/components/ui/badge.tsx` tiene 7 variantes. Las semánticas
(`success`/`warning`/`destructive`) siguen el mismo patrón visual: fondo del
color al 10% + texto del color puro (pill, no relleno sólido).

- **`success`** — algo se completó bien: venta completada, compra registrada,
  producto/proveedor activo, importación: fila creada, caja abierta.
- **`warning`** — necesita atención pero no es un error: stock bajo (>0),
  producto sin movimiento, toma de inventario abierta (en curso), fila
  omitida en una importación.
- **`destructive`** — error o algo irreversible: stock en 0 (agotado), venta
  o compra anulada, fila eliminada, fila con error de importación.
- **`secondary`** — neutral, ni bueno ni malo: producto/proveedor inactivo,
  caja cerrada, acción "modificar" en el historial de auditoría.
- **`outline`** — etiqueta informativa sin carga de estado: modo de
  importación, tipo de respaldo (manual/automático).
- **`default`** — reservado para cuando el color del texto/fondo debe ser el
  acento (`--primary`) y no un estado; en la práctica casi todo terminó en
  `success`/`warning`/`destructive`/`secondary` porque casi todo lo que se
  etiquetaba en la app es, en el fondo, un estado.

Antes de agregar un badge nuevo: preguntarse si representa un estado
(success/warning/destructive) o es solo una etiqueta neutral (outline/secondary).

## Tipografía

- **Poppins** (pesos 500/600/700, self-hosted vía `@fontsource/poppins`,
  solo subconjuntos `latin`/`latin-ext` para no inflar el instalador) —
  títulos (`h1`, `CardTitle` vía la clase `font-heading`), nav, marca.
- **Geist Variable** (ya estaba desde la fase 1) — todo el resto del texto.

Cualquier título nuevo (`h1`, `h2`, nombre de sección) debería llevar la clase
`font-heading` para heredar Poppins; `CardTitle` ya la trae por defecto.

## Sidebar

El ítem de menú activo usa `bg-primary/10 text-primary` (tinte, no relleno
sólido) — ver `src/layout/AppShell.tsx`. Es la misma idea que los badges de
estado: color de acento como tinte de fondo + texto del mismo color, nunca
fondo sólido con texto blanco salvo en botones primarios.

## Qué no cambió con este rediseño

- **Comprobantes PDF/PNG** (`src-tauri/src/pdf/`) — pipeline de Rust
  independiente, con su propio estilo de recibo. No usa estas variables.
- **Ícono de la app** (`src-tauri/icons/`) — sigue siendo el ícono simple
  "CF" original.

## Cómo verificar cambios de diseño sin Tauri

No hay forma de abrir una ventana real en este entorno de desarrollo
(sandbox sin gestor de ventanas), pero sí se puede ver el resultado real:

```bash
pnpm build
npx vite preview --port 4321 --strictPort &
npx playwright screenshot --browser cr --channel chrome \
  --color-scheme=light --viewport-size=1280,860 \
  "http://localhost:4321/" captura.png
```

Para tema oscuro, agregar `--color-scheme=dark` (el toggle de la app usa un
valor guardado en la configuración, no `prefers-color-scheme`, así que el
`--color-scheme` del navegador no alcanza solo — hay que forzar la clase
`.dark` en `<html>` o precargar `localStorage["theme"]="dark"` con
`--load-storage`). Las páginas que dependen de datos van a quedarse en
"Cargando…" porque no hay backend Tauri detrás de `vite preview` — sirve para
revisar layout, color y tipografía, no para probar funcionalidad.

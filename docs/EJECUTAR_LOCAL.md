# Ejecutar CajaFácil en local

Guía paso a paso para levantar el proyecto en tu máquina, sea para desarrollar
o solo para probarlo. Para instrucciones de instalación del producto final
(el `.msi`/`.exe`), ver el [README](../README.md#generar-el-instalador-de-windows).

## 1. Requisitos previos

- **[Node.js](https://nodejs.org/) 22 o superior** y **[pnpm](https://pnpm.io/) 10**.
  ```bash
  corepack enable        # si no tenés pnpm instalado
  corepack prepare pnpm@10 --activate
  ```
- **[Rust](https://www.rust-lang.org/tools/install)** (toolchain estable) vía `rustup`.
  Tauri compila un binario nativo, así que necesita el compilador de Rust
  aunque no vayas a tocar el backend.
- **Según tu sistema operativo:**

  | SO      | Qué falta                                                                                                                                                                                             |
  | ------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
  | Windows | [Visual Studio Build Tools](https://tauri.app/start/prerequisites/#windows) con el workload "Desktop development with C++". WebView2 se instala solo si falta.                                        |
  | Linux   | Librerías de desarrollo de WebKitGTK (ver comando abajo). Sirve para desarrollar y correr `pnpm tauri dev`, pero **el instalador final (.msi/.exe) no se puede generar desde Linux** — ver sección 5. |
  | macOS   | Xcode Command Line Tools (`xcode-select --install`). No es la plataforma objetivo del proyecto, pero `pnpm tauri dev` debería funcionar para desarrollar.                                             |

  Linux:

  ```bash
  sudo apt install libwebkit2gtk-4.1-dev libglib2.0-dev libgtk-3-dev \
    libsoup-3.0-dev build-essential libxdo-dev libssl-dev \
    libayatana-appindicator3-dev librsvg2-dev
  ```

Verificá que quedó todo instalado:

```bash
node -v      # v22.x o superior
pnpm -v      # 10.x
rustc --version
cargo --version
```

## 2. Clonar e instalar dependencias

```bash
git clone <url-del-repositorio> CajaFacil
cd CajaFacil
pnpm install
```

Esto instala tanto las dependencias del frontend (`package.json`) como el
`@tauri-apps/cli`, que se usa como `pnpm tauri ...` en todos los comandos de
esta guía.

## 3. Levantar la app en modo desarrollo

```bash
pnpm tauri dev
```

La primera vez tarda varios minutos porque compila todas las dependencias de
Rust desde cero; las siguientes veces es mucho más rápido (compilación
incremental). Este comando:

1. Levanta Vite en modo desarrollo (recarga en caliente del frontend).
2. Compila el backend Rust en modo debug.
3. Abre la ventana de la aplicación apuntando al frontend de Vite.

Si solo querés trabajar en el frontend sin recompilar Rust cada vez, dejá
`pnpm tauri dev` corriendo en una terminal — los cambios en `src/` se recargan
solos; los cambios en `src-tauri/src/` sí recompilan el backend y reinician
la app automáticamente.

### Dónde quedan los datos

La base de datos SQLite, los logs, los comprobantes generados y los respaldos
se guardan **fuera de la carpeta del proyecto**, en el directorio de datos de
la aplicación:

- Windows: `%APPDATA%\CajaFacil\`
- Linux (para desarrollo): `~/.local/share/com.daniel.cajafacil/`
- macOS: `~/Library/Application Support/com.daniel.cajafacil/`

Dentro vas a encontrar:

```
cajafacil.sqlite3       Base de datos (se crea sola en el primer arranque)
logs/                    Logs técnicos diarios (tracing)
comprobantes/            PDFs/PNG de ventas, organizados por año/mes
importaciones/           Archivos de carga masiva y sus rechazados
respaldos/                Copias de la base de datos (fase 7)
```

**Para empezar de cero** (borrar todos los datos de prueba y que la app
recree la base de datos vacía en el próximo arranque): cerrá la app y borrá
esa carpeta completa. No hay ningún comando dentro del proyecto que la borre
por vos — es una acción manual e irreversible, así que asegurate de no tener
datos que quieras conservar.

### Si dejaste un PIN configurado y te lo olvidaste

No hay una forma de recuperarlo desde la UI (el PIN se guarda con hash
Argon2, no en texto plano). Para desarrollo, la salida rápida es borrar el
directorio de datos como se explicó arriba. En un caso real de producción,
habría que editar la base de datos directamente (`UPDATE empresa_config SET
pin_hash = NULL, requiere_pin = 0 WHERE id = 1`) con un cliente SQLite.

## 4. Pruebas y linters

Antes de dar por buena cualquier corrida local, conviene dejar esto en verde:

```bash
# Backend
cd src-tauri
cargo test
cargo clippy --all-targets -- -D warnings
cargo fmt --check

# Frontend (desde la raíz del proyecto)
cd ..
pnpm test           # Vitest
pnpm lint           # eslint + prettier --check
npx tsc --noEmit    # chequeo de tipos
```

## 5. Generar el instalador de Windows

El instalador (`.msi` vía WiX y `.exe` vía NSIS) **solo se puede compilar
desde Windows** — WiX y NSIS no cross-compilan desde Linux ni macOS.

```powershell
pnpm install
pnpm tauri build
```

Los instaladores quedan en:

- `src-tauri/target/release/bundle/msi/*.msi`
- `src-tauri/target/release/bundle/nsis/*.exe`

Si estás en Linux y necesitás el instalador de Windows, usá el workflow de
GitHub Actions (`.github/workflows/build-windows.yml`, se dispara manualmente
desde la pestaña Actions del repositorio) en vez de intentar compilarlo local.

## 6. Problemas comunes

| Síntoma                                              | Causa probable                                                                                                                                                                                                                                         |
| ---------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| `error: linker \`cc\` not found` (Linux)             | Falta `build-essential`. Instalá el paquete del punto 1.                                                                                                                                                                                               |
| La ventana no abre / queda en blanco                 | Puede ser WebKitGTK faltante o desactualizado en Linux. Revisá que las librerías del punto 1 estén instaladas.                                                                                                                                         |
| `pnpm tauri dev` tarda "para siempre" la primera vez | Es normal — está compilando ~600 crates de Rust desde cero. Las siguientes corridas son incrementales.                                                                                                                                                 |
| Cambios en `src-tauri/src/` no se reflejan           | Esperá a que termine de recompilar (mirá la terminal); si quedó colgado, cerrá y volvé a correr `pnpm tauri dev`.                                                                                                                                      |
| "database is locked" en los tests de Rust            | El pool de conexiones SQLite del proyecto usa `max_size = 1` a propósito (ver comentario en `src-tauri/src/db/mod.rs`). Si estás escribiendo un test nuevo, asegurate de soltar (`drop(conn)`) cualquier conexión abierta antes de pedir otra al pool. |
| Botón "Buscar actualizaciones" siempre falla         | Esperado — todavía no hay un servidor de actualizaciones configurado. Ver [README → Publicar actualizaciones](../README.md#publicar-actualizaciones).                                                                                                  |

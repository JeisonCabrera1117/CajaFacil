# Pendientes

Las 7 fases del plan original están completas (ver "Estado del proyecto" en
el [README](../README.md)) y todo lo construido está verificado con tests,
linters y (para la fase 7 y el rediseño) capturas reales del build. Esto no
es una lista de bugs — es lo que falta para pasar de "app funcional
verificada en Linux/dev" a "instalador que le vas a dar a un usuario real en
Windows".

## Antes de lo demás: no hay ningún commit

```
$ git log
fatal: tu rama actual 'master' no tiene ningún commit todavía
```

Todo el proyecto (7 fases + rediseño) existe solo como archivos sin
versionar y sin repositorio remoto. Es lo primero que hay que resolver —
cualquier otro pendiente de esta lista es menos urgente que el riesgo de
perder el trabajo por no tener ni un commit hecho.

## Para el primer instalador real

- **Probar en Windows de verdad.** Todo se compiló y se corrió bajo Linux
  (Xvfb para el binario, `vite preview` + Chrome headless para la UI). Nunca
  se abrió una ventana real de la app. `pnpm tauri build` **solo se puede
  ejecutar desde Windows** (WiX/NSIS no compilan cruzado desde Linux) — hasta
  que alguien lo corra ahí, o vía el workflow de GitHub Actions, no hay
  instalador generado en absoluto.
- **Revisar `tauri.conf.json`** antes de repartirlo: nombre/versión (sigue en
  `0.1.0`), y el ícono (sigue siendo el simple "CF" de la fase 1, sin el
  rediseño de colores — ver `docs/DISENO.md`).
- **Firma de código del instalador** (distinta de la firma del updater): no
  hay certificado configurado. Sin uno, Windows va a mostrar la advertencia
  de "editor desconocido" al instalar. `certificateThumbprint` en
  `tauri.conf.json` está listo para recibirlo cuando exista.

## Actualizaciones automáticas

Wireado (`tauri-plugin-updater`, botón en Configuración) pero no
funcional todavía porque:

- El endpoint en `tauri.conf.json` → `plugins.updater.endpoints` es un
  placeholder (`https://TODO-configurar-hosting...`). Falta decidir dónde
  alojar releases (GitHub Releases es lo más simple) y publicar ahí el
  `latest.json` que genera `tauri build`.
- La clave privada de firma (`~/claves-cajafacil/cajafacil-updater.key`) se
  generó **con contraseña vacía** para no bloquear el desarrollo. Antes de
  publicar una actualización real: regenerarla con contraseña, actualizar el
  secreto `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` en GitHub y el `pubkey` en
  `tauri.conf.json`.
- Pasos completos en el README, sección "Publicar actualizaciones".

## Respaldos: solo manuales

La tabla `respaldos` y el tipo en frontend distinguen `"manual"` vs.
`"automatico"`, pero **nada crea un respaldo automático todavía** — es un
campo que existe en el esquema y no se usa. Si se quiere un respaldo
periódico sin que el usuario tenga que acordarse de apretar el botón, falta
implementarlo (candidato natural: al arrancar la app, si el último respaldo
tiene más de N días, crear uno con `tipo: "automatico"` antes de abrir la
ventana).

## Diseño: lo que quedó fuera del rediseño estilo monday.com

Ver `docs/DISENO.md` para el detalle completo. Resumen de lo pendiente:

- **Comprobantes PDF/PNG** (`src-tauri/src/pdf/`): pipeline de Rust aparte,
  sigue con su estilo original — no hereda la paleta nueva porque no usa las
  variables CSS del frontend.
- **Ícono de la app**: no se tocó.
- No se hizo una revisión visual pantalla por pantalla (solo se capturaron
  Dashboard, Inventario y Reportes en claro/oscuro). El resto hereda el
  mismo sistema de tokens, pero nadie lo vio renderizado todavía.

## Deuda menor / cosas que se notaron pero no se atacaron

- `dist/assets/index-*.js` pesa ~1.2 MB (viene marcándolo Vite desde antes
  del rediseño) — candidato a code-splitting si el arranque en frío llega a
  sentirse lento en un equipo modesto.
- `next-themes` está con `enableSystem={false}` y `defaultTheme="light"` a
  propósito (el tema lo controla el campo `tema` guardado en la app, no el
  SO) — no es un bug, pero puede sorprender si alguien espera que siga el
  tema del sistema operativo.

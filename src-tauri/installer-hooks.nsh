; Hooks del instalador NSIS para CajaFácil.
; Referenciado desde tauri.conf.json -> bundle.windows.nsis.installerHooks

!macro NSIS_HOOK_POSTUNINSTALL
  MessageBox MB_YESNO|MB_ICONQUESTION \
    "¿Desea eliminar también los datos guardados de CajaFácil (base de datos, comprobantes y respaldos)?$\r$\n$\r$\nSi elige 'No', sus datos quedarán intactos en:$\r$\n$APPDATA\CajaFacil" \
    IDYES borrar_datos IDNO conservar_datos

  borrar_datos:
    RMDir /r "$APPDATA\CajaFacil"
    Goto fin_datos

  conservar_datos:
    Goto fin_datos

  fin_datos:
!macroend

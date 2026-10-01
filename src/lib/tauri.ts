import { invoke } from "@tauri-apps/api/core";

export interface EmpresaConfig {
  nombre: string;
  nit: string;
  direccion: string;
  telefono: string;
  logoPath: string | null;
  moneda: string;
  formatoFecha: string;
  prefijoComprobante: string;
  siguienteNumero: number;
  leyendaPie: string;
  tema: string;
  requierePin: boolean;
  permiteStockNegativo: boolean;
  arqueoActivo: boolean;
}

export interface EmpresaConfigActualizar {
  nombre: string;
  nit: string;
  direccion: string;
  telefono: string;
  logoPath: string | null;
  moneda: string;
  formatoFecha: string;
  prefijoComprobante: string;
  leyendaPie: string;
  tema: string;
  permiteStockNegativo: boolean;
  arqueoActivo: boolean;
}

/**
 * Wrappers tipados sobre los comandos de Tauri. Los errores llegan como
 * string (mensaje ya listo para mostrar) porque el backend serializa
 * AppError como un mensaje amigable en español.
 */
export const api = {
  authCheckRequired: () => invoke<boolean>("auth_check_required"),
  authSetPin: (pin: string) => invoke<void>("auth_set_pin", { pin }),
  authVerifyPin: (pin: string) => invoke<boolean>("auth_verify_pin", { pin }),
  authDisablePin: (pinActual: string) => invoke<void>("auth_disable_pin", { pinActual }),

  configGet: () => invoke<EmpresaConfig>("config_get"),
  configUpdate: (datos: EmpresaConfigActualizar) =>
    invoke<EmpresaConfig>("config_update", { datos }),
};

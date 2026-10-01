import { invoke } from "@tauri-apps/api/core";

export type TipoComprobante = "carta" | "termico80" | "imagen";

export interface Comprobante {
  id: number;
  ventaId: number;
  tipo: TipoComprobante;
  rutaArchivo: string;
  generadoEn: string;
}

export const apiComprobantes = {
  generar: (ventaId: number, formato: TipoComprobante) =>
    invoke<Comprobante>("comprobante_generar", { ventaId, formato }),
  regenerar: (ventaId: number, formato: TipoComprobante) =>
    invoke<Comprobante>("comprobante_regenerar", { ventaId, formato }),
  abrir: (rutaArchivo: string) => invoke<void>("comprobante_abrir", { rutaArchivo }),
  copiarImagen: (rutaArchivo: string) => invoke<void>("comprobante_copiar_imagen", { rutaArchivo }),
};

export const ETIQUETA_TIPO_COMPROBANTE: Record<TipoComprobante, string> = {
  carta: "PDF carta",
  termico80: "PDF térmico 80mm",
  imagen: "Imagen (PNG)",
};

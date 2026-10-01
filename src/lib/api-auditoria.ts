import { invoke } from "@tauri-apps/api/core";

export interface EntradaAuditoria {
  id: number;
  fecha: string;
  entidad: string;
  entidadId: number | null;
  accion: "crear" | "modificar" | "anular" | "eliminar";
  detalleJson: string | null;
}

export interface FiltroAuditoria {
  entidad?: string | null;
  desde?: string | null;
  hasta?: string | null;
  pagina?: number;
  porPagina?: number;
}

export const ENTIDADES_AUDITORIA = [
  "producto",
  "categoria",
  "proveedor",
  "cliente",
  "compra",
  "venta",
  "devolucion",
  "comprobante",
  "caja_sesion",
  "inventario_fisico",
  "importacion",
] as const;

export const apiAuditoria = {
  list: (filtro: FiltroAuditoria) => invoke<EntradaAuditoria[]>("auditoria_list", { filtro }),
};

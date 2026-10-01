import { invoke } from "@tauri-apps/api/core";

export interface Respaldo {
  id: number;
  fecha: string;
  rutaArchivo: string;
  tipo: "manual" | "automatico";
  tamanoBytes: number;
}

export const apiRespaldos = {
  list: () => invoke<Respaldo[]>("respaldo_list"),
  crear: () => invoke<Respaldo>("respaldo_crear"),
  eliminar: (id: number) => invoke<void>("respaldo_eliminar", { id }),
  restaurar: (id: number) => invoke<void>("respaldo_restaurar", { id }),
};

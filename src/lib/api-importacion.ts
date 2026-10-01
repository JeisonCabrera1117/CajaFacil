import { invoke } from "@tauri-apps/api/core";

export type ModoImport = "crear" | "actualizar" | "crear_y_actualizar";

export interface EntidadImportable {
  id: string;
  etiqueta: string;
}

export interface CampoImport {
  id: string;
  etiqueta: string;
  obligatorio: boolean;
}

export interface DeteccionArchivo {
  tipo: "csv" | "xlsx";
  encoding: string | null;
  separador: string | null;
  hojas: string[];
  hojaSeleccionada: string | null;
  columnas: string[];
  filasMuestra: string[][];
  totalFilas: number;
}

export interface FilaRechazada {
  fila: number;
  motivo: string;
}

export interface ResumenImportacion {
  totalFilas: number;
  creados: number;
  actualizados: number;
  omitidos: number;
  conError: number;
  filasRechazadas: FilaRechazada[];
}

export interface Importacion {
  id: number;
  entidad: string;
  archivoNombre: string;
  fecha: string;
  modo: string;
  totalFilas: number;
  creados: number;
  actualizados: number;
  omitidos: number;
  conError: number;
  archivoRechazadosPath: string | null;
}

export interface PlantillaGenerada {
  nombreArchivo: string;
  contenidoBase64: string;
  mime: string;
}

export interface ProgresoImport {
  procesadas: number;
  total: number;
}

export const apiImportacion = {
  entidades: () => invoke<EntidadImportable[]>("import_entidades"),
  campos: (entidad: string) => invoke<CampoImport[]>("import_campos", { entidad }),
  plantillaDescargar: (entidad: string, formato: "csv" | "xlsx") =>
    invoke<PlantillaGenerada>("import_plantilla_descargar", { entidad, formato }),
  detectarArchivo: (ruta: string, hoja?: string | null, separador?: string | null) =>
    invoke<DeteccionArchivo>("import_detectar_archivo", {
      ruta,
      hoja: hoja ?? null,
      separador: separador ?? null,
    }),
  preview: (params: {
    ruta: string;
    hoja: string | null;
    separador: string | null;
    entidad: string;
    mapeo: Record<string, string>;
    modo: ModoImport;
  }) => invoke<ResumenImportacion>("import_preview", params),
  ejecutar: (params: {
    ruta: string;
    hoja: string | null;
    separador: string | null;
    entidad: string;
    mapeo: Record<string, string>;
    modo: ModoImport;
    nombreArchivoOriginal: string;
  }) => invoke<Importacion>("import_ejecutar", params),
  historial: () => invoke<Importacion[]>("import_historial"),
};

/** Dispara la descarga de una plantilla ya generada (base64) como archivo real del navegador. */
export function descargarPlantilla(plantilla: PlantillaGenerada) {
  const bytes = atob(plantilla.contenidoBase64);
  const arreglo = new Uint8Array(bytes.length);
  for (let i = 0; i < bytes.length; i++) arreglo[i] = bytes.charCodeAt(i);
  const blob = new Blob([arreglo], { type: plantilla.mime });
  const url = URL.createObjectURL(blob);
  const enlace = document.createElement("a");
  enlace.href = url;
  enlace.download = plantilla.nombreArchivo;
  document.body.appendChild(enlace);
  enlace.click();
  enlace.remove();
  URL.revokeObjectURL(url);
}

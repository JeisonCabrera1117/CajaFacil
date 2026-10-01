import { format as formatDate } from "date-fns";
import { es } from "date-fns/locale";

/** Los montos siempre llegan del backend como enteros (centavos). */
export function formatearMoneda(centavos: number, moneda: string = "COP"): string {
  const valor = centavos / 100;
  const sinDecimales = moneda === "COP";
  return new Intl.NumberFormat("es-CO", {
    style: "currency",
    currency: moneda,
    minimumFractionDigits: sinDecimales ? 0 : 2,
    maximumFractionDigits: sinDecimales ? 0 : 2,
  }).format(valor);
}

const MAPA_FORMATOS_FECHA: Record<string, string> = {
  "DD/MM/YYYY": "dd/MM/yyyy",
  "MM/DD/YYYY": "MM/dd/yyyy",
  "YYYY-MM-DD": "yyyy-MM-dd",
};

export function formatearFecha(fechaIso: string, formato: string): string {
  const patron = MAPA_FORMATOS_FECHA[formato] ?? "dd/MM/yyyy";
  return formatDate(new Date(fechaIso), patron, { locale: es });
}

const MAPA_FORMATOS_FECHA_HORA: Record<string, string> = {
  "DD/MM/YYYY": "dd/MM/yyyy HH:mm",
  "MM/DD/YYYY": "MM/dd/yyyy HH:mm",
  "YYYY-MM-DD": "yyyy-MM-dd HH:mm",
};

/**
 * El backend guarda timestamps como "YYYY-MM-DD HH:MM:SS" en UTC (ver
 * docs/ARQUITECTURA.md). `new Date()` interpreta esos mismos dígitos como si
 * ya fueran hora local si no se lo decimos explícito — por eso el reemplazo
 * de espacio por "T" + "Z" antes de parsear.
 */
export function formatearFechaHora(fechaSqliteUtc: string, formato: string = "DD/MM/YYYY"): string {
  const patron = MAPA_FORMATOS_FECHA_HORA[formato] ?? MAPA_FORMATOS_FECHA_HORA["DD/MM/YYYY"];
  const fecha = new Date(fechaSqliteUtc.replace(" ", "T") + "Z");
  return formatDate(fecha, patron, { locale: es });
}

export function hoyISO(): string {
  return formatDate(new Date(), "yyyy-MM-dd");
}

export function primerDiaMesISO(): string {
  const ahora = new Date();
  return formatDate(new Date(ahora.getFullYear(), ahora.getMonth(), 1), "yyyy-MM-dd");
}

export function formatearTamano(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  const kb = bytes / 1024;
  if (kb < 1024) return `${kb.toFixed(1)} KB`;
  return `${(kb / 1024).toFixed(1)} MB`;
}

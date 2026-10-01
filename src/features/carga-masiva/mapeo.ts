import type { CampoImport } from "@/lib/api-importacion";

function normalizar(s: string): string {
  return s
    .normalize("NFD")
    .replace(/[̀-ͯ]/g, "")
    .toLowerCase()
    .replace(/[^a-z0-9]/g, "");
}

/** Sugiere, para cada campo del sistema, la columna del archivo cuyo nombre se le parece más. */
export function sugerirMapeo(campos: CampoImport[], columnas: string[]): Record<string, string> {
  const columnasNormalizadas = columnas.map((c) => ({ original: c, normalizada: normalizar(c) }));
  const mapeo: Record<string, string> = {};

  for (const campo of campos) {
    const campoNorm = normalizar(campo.id);
    const etiquetaNorm = normalizar(campo.etiqueta);

    const exacto = columnasNormalizadas.find(
      (c) => c.normalizada === campoNorm || c.normalizada === etiquetaNorm,
    );
    if (exacto) {
      mapeo[campo.id] = exacto.original;
      continue;
    }

    const parcial = columnasNormalizadas.find(
      (c) => c.normalizada.includes(campoNorm) || campoNorm.includes(c.normalizada),
    );
    if (parcial) {
      mapeo[campo.id] = parcial.original;
    }
  }

  return mapeo;
}

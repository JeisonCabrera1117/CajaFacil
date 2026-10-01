/** Orden categórico fijo (no ciclado) tomado de las variables de tema ya
 * definidas para gráficas (`--chart-1`..`--chart-5` en index.css, con su
 * propia versión para modo oscuro). Nunca generar un color nuevo para una
 * 6ta serie: si hay más categorías, se agrupan en "Otros". */
export const PALETA_GRAFICAS = [
  "var(--chart-1)",
  "var(--chart-2)",
  "var(--chart-3)",
  "var(--chart-4)",
  "var(--chart-5)",
];

export const MAX_SEGMENTOS = PALETA_GRAFICAS.length;

/** Agrupa los elementos que sobran del top N en un segmento "Otros". */
export function agruparEnOtros<T extends { total: number }>(
  items: T[],
  maximo: number = MAX_SEGMENTOS,
): (T | { etiqueta: string; total: number })[] {
  if (items.length <= maximo) return items;
  const visibles = items.slice(0, maximo - 1);
  const resto = items.slice(maximo - 1).reduce((acc, i) => acc + i.total, 0);
  return [...visibles, { etiqueta: "Otros", total: resto }];
}

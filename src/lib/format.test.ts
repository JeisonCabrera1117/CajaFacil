import { describe, expect, it } from "vitest";
import { format as formatDate } from "date-fns";
import { es } from "date-fns/locale";
import { formatearMoneda, formatearFecha, formatearFechaHora } from "@/lib/format";

// Intl separa simbolo y monto con un espacio de no separacion; lo normalizamos
// para no acoplar el test a ese detalle de formato.
const ESPACIO_NO_SEPARABLE = String.fromCharCode(160);

function sinEspaciosEspeciales(texto: string): string {
  return texto.split(ESPACIO_NO_SEPARABLE).join(" ");
}

describe("formatearMoneda", () => {
  it("formatea pesos colombianos sin decimales", () => {
    expect(sinEspaciosEspeciales(formatearMoneda(123456700, "COP"))).toBe("$ 1.234.567");
  });

  it("formatea otras monedas con dos decimales", () => {
    expect(sinEspaciosEspeciales(formatearMoneda(150050, "USD"))).toBe("US$ 1.500,50");
  });
});

describe("formatearFecha", () => {
  it("respeta el formato DD/MM/YYYY", () => {
    expect(formatearFecha("2026-03-05T00:00:00", "DD/MM/YYYY")).toBe("05/03/2026");
  });

  it("respeta el formato YYYY-MM-DD", () => {
    expect(formatearFecha("2026-03-05T00:00:00", "YYYY-MM-DD")).toBe("2026-03-05");
  });
});

describe("formatearFechaHora", () => {
  // El esperado se calcula convirtiendo el mismo instante UTC a hora local
  // (en vez de codificar una hora fija), para que el test no dependa del
  // huso horario de la máquina donde corre.
  const instanteUtc = new Date("2026-03-05T14:32:10Z");

  it("convierte de UTC a hora local, no muestra los dígitos crudos", () => {
    expect(formatearFechaHora("2026-03-05 14:32:10", "DD/MM/YYYY")).toBe(
      formatDate(instanteUtc, "dd/MM/yyyy HH:mm", { locale: es }),
    );
  });

  it("respeta el formato YYYY-MM-DD", () => {
    expect(formatearFechaHora("2026-03-05 14:32:10", "YYYY-MM-DD")).toBe(
      formatDate(instanteUtc, "yyyy-MM-dd HH:mm", { locale: es }),
    );
  });

  it("usa DD/MM/YYYY por defecto si no se pasa formato", () => {
    expect(formatearFechaHora("2026-03-05 14:32:10")).toBe(
      formatDate(instanteUtc, "dd/MM/yyyy HH:mm", { locale: es }),
    );
  });
});

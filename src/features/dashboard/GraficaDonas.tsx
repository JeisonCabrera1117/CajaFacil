import { useRef } from "react";
import { Cell, Legend, Pie, PieChart, ResponsiveContainer, Tooltip } from "recharts";
import type { PuntoSerie } from "@/lib/api-dashboard";
import { formatearMoneda } from "@/lib/format";
import { exportarGraficaComoPng } from "@/lib/exportar-grafica";
import { agruparEnOtros, PALETA_GRAFICAS } from "@/features/dashboard/paleta";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";

function GraficaDona({ titulo, datos }: { titulo: string; datos: PuntoSerie[] }) {
  const contenedorRef = useRef<HTMLDivElement>(null);
  const agrupados = agruparEnOtros(datos);

  return (
    <Card>
      <CardHeader className="flex flex-row items-center justify-between space-y-0">
        <CardTitle className="text-base">{titulo}</CardTitle>
        <Button
          variant="outline"
          size="sm"
          onClick={() => exportarGraficaComoPng(contenedorRef.current, titulo)}
        >
          PNG
        </Button>
      </CardHeader>
      <CardContent>
        <div ref={contenedorRef} className="h-72 w-full">
          {agrupados.length === 0 ? (
            <p className="flex h-full items-center justify-center text-sm text-muted-foreground">
              Sin datos en este rango.
            </p>
          ) : (
            <ResponsiveContainer width="100%" height="100%">
              <PieChart>
                <Pie
                  data={agrupados}
                  dataKey="total"
                  nameKey="etiqueta"
                  innerRadius="55%"
                  outerRadius="80%"
                  paddingAngle={2}
                >
                  {agrupados.map((entrada, i) => (
                    <Cell
                      key={entrada.etiqueta}
                      fill={PALETA_GRAFICAS[i % PALETA_GRAFICAS.length]}
                    />
                  ))}
                </Pie>
                <Legend wrapperStyle={{ fontSize: 12, color: "var(--muted-foreground)" }} />
                <Tooltip
                  formatter={(valor) => formatearMoneda(Number(valor))}
                  contentStyle={{
                    backgroundColor: "var(--popover)",
                    color: "var(--popover-foreground)",
                    border: "1px solid var(--border)",
                    borderRadius: "var(--radius-md)",
                  }}
                />
              </PieChart>
            </ResponsiveContainer>
          )}
        </div>
      </CardContent>
    </Card>
  );
}

export function GraficaDonas({
  porCategoria,
  porMetodoPago,
}: {
  porCategoria: PuntoSerie[];
  porMetodoPago: PuntoSerie[];
}) {
  return (
    <div className="grid gap-4 lg:grid-cols-2">
      <GraficaDona titulo="Ventas por categoría" datos={porCategoria} />
      <GraficaDona titulo="Ventas por método de pago" datos={porMetodoPago} />
    </div>
  );
}

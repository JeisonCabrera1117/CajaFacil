import { useRef } from "react";
import { Bar, BarChart, CartesianGrid, ResponsiveContainer, Tooltip, XAxis, YAxis } from "recharts";
import type { ProductoCantidad } from "@/lib/api-dashboard";
import { exportarGraficaComoPng } from "@/lib/exportar-grafica";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";

function GraficaBarras({
  titulo,
  datos,
  color,
}: {
  titulo: string;
  datos: ProductoCantidad[];
  color: string;
}) {
  const contenedorRef = useRef<HTMLDivElement>(null);

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
          {datos.length === 0 ? (
            <p className="flex h-full items-center justify-center text-sm text-muted-foreground">
              Sin ventas en este rango.
            </p>
          ) : (
            <ResponsiveContainer width="100%" height="100%">
              <BarChart
                data={datos}
                layout="vertical"
                margin={{ top: 8, right: 16, bottom: 0, left: 8 }}
              >
                <CartesianGrid strokeDasharray="3 3" stroke="var(--border)" horizontal={false} />
                <XAxis
                  type="number"
                  tick={{ fontSize: 12, fill: "var(--muted-foreground)" }}
                  allowDecimals={false}
                />
                <YAxis
                  type="category"
                  dataKey="nombre"
                  width={110}
                  tick={{ fontSize: 11, fill: "var(--muted-foreground)" }}
                />
                <Tooltip
                  contentStyle={{
                    backgroundColor: "var(--popover)",
                    color: "var(--popover-foreground)",
                    border: "1px solid var(--border)",
                    borderRadius: "var(--radius-md)",
                  }}
                />
                <Bar dataKey="cantidad" fill={color} radius={[0, 4, 4, 0]} />
              </BarChart>
            </ResponsiveContainer>
          )}
        </div>
      </CardContent>
    </Card>
  );
}

export function GraficaProductos({
  masVendidos,
  menosVendidos,
}: {
  masVendidos: ProductoCantidad[];
  menosVendidos: ProductoCantidad[];
}) {
  return (
    <div className="grid gap-4 lg:grid-cols-2">
      <GraficaBarras titulo="Productos más vendidos" datos={masVendidos} color="var(--chart-1)" />
      <GraficaBarras
        titulo="Productos menos vendidos"
        datos={menosVendidos}
        color="var(--chart-2)"
      />
    </div>
  );
}

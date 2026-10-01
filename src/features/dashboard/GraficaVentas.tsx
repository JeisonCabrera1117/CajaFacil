import { useRef } from "react";
import {
  CartesianGrid,
  Line,
  LineChart,
  ResponsiveContainer,
  Tooltip,
  XAxis,
  YAxis,
} from "recharts";
import type { PuntoSerie } from "@/lib/api-dashboard";
import { formatearMoneda } from "@/lib/format";
import { exportarGraficaComoPng } from "@/lib/exportar-grafica";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";

interface Props {
  serie: PuntoSerie[];
  desde: string;
  hasta: string;
  onDesdeChange: (v: string) => void;
  onHastaChange: (v: string) => void;
}

export function GraficaVentas({ serie, desde, hasta, onDesdeChange, onHastaChange }: Props) {
  const contenedorRef = useRef<HTMLDivElement>(null);

  return (
    <Card>
      <CardHeader className="flex flex-row flex-wrap items-center justify-between gap-3 space-y-0">
        <CardTitle>Ventas por día</CardTitle>
        <div className="flex flex-wrap items-end gap-2">
          <div className="space-y-1">
            <Label className="text-xs">Desde</Label>
            <Input
              type="date"
              className="h-8"
              value={desde}
              onChange={(e) => onDesdeChange(e.currentTarget.value)}
            />
          </div>
          <div className="space-y-1">
            <Label className="text-xs">Hasta</Label>
            <Input
              type="date"
              className="h-8"
              value={hasta}
              onChange={(e) => onHastaChange(e.currentTarget.value)}
            />
          </div>
          <Button
            variant="outline"
            size="sm"
            onClick={() => exportarGraficaComoPng(contenedorRef.current, "ventas-por-dia")}
          >
            Exportar PNG
          </Button>
        </div>
      </CardHeader>
      <CardContent>
        <div ref={contenedorRef} className="h-72 w-full">
          {serie.length === 0 ? (
            <p className="flex h-full items-center justify-center text-sm text-muted-foreground">
              Sin ventas en este rango.
            </p>
          ) : (
            <ResponsiveContainer width="100%" height="100%">
              <LineChart data={serie} margin={{ top: 8, right: 16, bottom: 0, left: 0 }}>
                <CartesianGrid strokeDasharray="3 3" stroke="var(--border)" vertical={false} />
                <XAxis
                  dataKey="etiqueta"
                  tick={{ fontSize: 12, fill: "var(--muted-foreground)" }}
                />
                <YAxis
                  tick={{ fontSize: 12, fill: "var(--muted-foreground)" }}
                  tickFormatter={(v) => formatearMoneda(v)}
                  width={90}
                />
                <Tooltip
                  formatter={(valor) => formatearMoneda(Number(valor))}
                  contentStyle={{
                    backgroundColor: "var(--popover)",
                    color: "var(--popover-foreground)",
                    border: "1px solid var(--border)",
                    borderRadius: "var(--radius-md)",
                  }}
                />
                <Line
                  type="monotone"
                  dataKey="total"
                  stroke="var(--chart-1)"
                  strokeWidth={2}
                  dot={{ r: 3 }}
                  activeDot={{ r: 5 }}
                />
              </LineChart>
            </ResponsiveContainer>
          )}
        </div>
      </CardContent>
    </Card>
  );
}

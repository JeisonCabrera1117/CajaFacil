import { useState } from "react";
import { useQuery } from "@tanstack/react-query";
import { apiReportes } from "@/lib/api-reportes";
import { formatearMoneda, primerDiaMesISO, hoyISO } from "@/lib/format";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import { ExportarReporteBotones } from "@/features/reportes/ExportarReporteBotones";

const COLUMNAS = ["Producto", "Cantidad vendida", "Ingresos", "Costo", "Utilidad"];

export function ReporteUtilidadPanel() {
  const [desde, setDesde] = useState(primerDiaMesISO());
  const [hasta, setHasta] = useState(hoyISO());

  const { data: filas, isLoading } = useQuery({
    queryKey: ["reporte-utilidad", desde, hasta],
    queryFn: () => apiReportes.utilidad(desde, hasta),
  });

  const totalUtilidad = (filas ?? []).reduce((acc, f) => acc + f.utilidad, 0);
  const filasExportar = (filas ?? []).map((f) => [
    f.producto,
    String(f.cantidadVendida),
    formatearMoneda(f.ingresos),
    formatearMoneda(f.costo),
    formatearMoneda(f.utilidad),
  ]);

  return (
    <div className="space-y-4">
      <div className="flex flex-wrap items-end gap-3">
        <div className="space-y-1.5">
          <Label>Desde</Label>
          <Input type="date" value={desde} onChange={(e) => setDesde(e.currentTarget.value)} />
        </div>
        <div className="space-y-1.5">
          <Label>Hasta</Label>
          <Input type="date" value={hasta} onChange={(e) => setHasta(e.currentTarget.value)} />
        </div>
        <div className="flex-1" />
        <p className="text-sm text-muted-foreground">
          Utilidad total:{" "}
          <span className="font-medium text-foreground">{formatearMoneda(totalUtilidad)}</span>
        </p>
        <ExportarReporteBotones
          titulo="Reporte de utilidad"
          columnas={COLUMNAS}
          filas={filasExportar}
        />
      </div>

      <p className="text-xs text-muted-foreground">
        La utilidad usa el costo actual del producto (no el costo histórico al momento de cada
        venta).
      </p>

      <div className="max-h-[28rem] overflow-y-auto rounded-md border border-border">
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead>Producto</TableHead>
              <TableHead className="text-right">Cant. vendida</TableHead>
              <TableHead className="text-right">Ingresos</TableHead>
              <TableHead className="text-right">Costo</TableHead>
              <TableHead className="text-right">Utilidad</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {isLoading && (
              <TableRow>
                <TableCell colSpan={5} className="text-center text-muted-foreground">
                  Cargando…
                </TableCell>
              </TableRow>
            )}
            {!isLoading && filas?.length === 0 && (
              <TableRow>
                <TableCell colSpan={5} className="text-center text-muted-foreground">
                  Sin ventas en este rango.
                </TableCell>
              </TableRow>
            )}
            {filas?.map((f, i) => (
              <TableRow key={i}>
                <TableCell>{f.producto}</TableCell>
                <TableCell className="text-right">{f.cantidadVendida}</TableCell>
                <TableCell className="text-right">{formatearMoneda(f.ingresos)}</TableCell>
                <TableCell className="text-right">{formatearMoneda(f.costo)}</TableCell>
                <TableCell className="text-right font-medium">
                  {formatearMoneda(f.utilidad)}
                </TableCell>
              </TableRow>
            ))}
          </TableBody>
        </Table>
      </div>
    </div>
  );
}

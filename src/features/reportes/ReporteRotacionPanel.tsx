import { useState } from "react";
import { useQuery } from "@tanstack/react-query";
import { apiReportes } from "@/lib/api-reportes";
import { primerDiaMesISO, hoyISO } from "@/lib/format";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Badge } from "@/components/ui/badge";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import { ExportarReporteBotones } from "@/features/reportes/ExportarReporteBotones";

const COLUMNAS = ["SKU", "Nombre", "Cantidad vendida", "Stock actual", "Sin movimiento"];

export function ReporteRotacionPanel() {
  const [desde, setDesde] = useState(primerDiaMesISO());
  const [hasta, setHasta] = useState(hoyISO());

  const { data: filas, isLoading } = useQuery({
    queryKey: ["reporte-rotacion", desde, hasta],
    queryFn: () => apiReportes.rotacion(desde, hasta),
  });

  const filasExportar = (filas ?? []).map((f) => [
    f.sku,
    f.nombre,
    String(f.cantidadVendida),
    String(f.stockActual),
    f.sinMovimiento ? "Sí" : "No",
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
        <ExportarReporteBotones
          titulo="Rotación de productos"
          columnas={COLUMNAS}
          filas={filasExportar}
        />
      </div>

      <div className="max-h-[28rem] overflow-y-auto rounded-md border border-border">
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead>SKU</TableHead>
              <TableHead>Nombre</TableHead>
              <TableHead className="text-right">Cant. vendida</TableHead>
              <TableHead className="text-right">Stock actual</TableHead>
              <TableHead>Sin movimiento</TableHead>
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
            {filas?.map((f) => (
              <TableRow key={f.sku}>
                <TableCell className="font-mono text-sm">{f.sku}</TableCell>
                <TableCell>{f.nombre}</TableCell>
                <TableCell className="text-right">{f.cantidadVendida}</TableCell>
                <TableCell className="text-right">{f.stockActual}</TableCell>
                <TableCell>
                  {f.sinMovimiento ? <Badge variant="warning">Sin movimiento</Badge> : "—"}
                </TableCell>
              </TableRow>
            ))}
          </TableBody>
        </Table>
      </div>
    </div>
  );
}

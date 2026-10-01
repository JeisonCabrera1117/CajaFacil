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

const COLUMNAS = ["Proveedor", "Número de compras", "Total comprado"];

export function ReporteComprasProveedorPanel() {
  const [desde, setDesde] = useState(primerDiaMesISO());
  const [hasta, setHasta] = useState(hoyISO());

  const { data: filas, isLoading } = useQuery({
    queryKey: ["reporte-compras-proveedor", desde, hasta],
    queryFn: () => apiReportes.comprasPorProveedor(desde, hasta),
  });

  const filasExportar = (filas ?? []).map((f) => [
    f.proveedor,
    String(f.numeroCompras),
    formatearMoneda(f.totalComprado),
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
          titulo="Compras por proveedor"
          columnas={COLUMNAS}
          filas={filasExportar}
        />
      </div>

      <div className="max-h-[28rem] overflow-y-auto rounded-md border border-border">
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead>Proveedor</TableHead>
              <TableHead className="text-right">Compras</TableHead>
              <TableHead className="text-right">Total comprado</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {isLoading && (
              <TableRow>
                <TableCell colSpan={3} className="text-center text-muted-foreground">
                  Cargando…
                </TableCell>
              </TableRow>
            )}
            {!isLoading && filas?.length === 0 && (
              <TableRow>
                <TableCell colSpan={3} className="text-center text-muted-foreground">
                  Sin compras en este rango.
                </TableCell>
              </TableRow>
            )}
            {filas?.map((f, i) => (
              <TableRow key={i}>
                <TableCell>{f.proveedor}</TableCell>
                <TableCell className="text-right">{f.numeroCompras}</TableCell>
                <TableCell className="text-right font-medium">
                  {formatearMoneda(f.totalComprado)}
                </TableCell>
              </TableRow>
            ))}
          </TableBody>
        </Table>
      </div>
    </div>
  );
}

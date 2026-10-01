import { useQuery } from "@tanstack/react-query";
import { apiReportes } from "@/lib/api-reportes";
import { formatearMoneda } from "@/lib/format";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import { ExportarReporteBotones } from "@/features/reportes/ExportarReporteBotones";

const COLUMNAS = ["SKU", "Nombre", "Categoría", "Stock actual", "Costo unitario", "Valor total"];

export function ReporteInventarioPanel() {
  const { data: filas, isLoading } = useQuery({
    queryKey: ["reporte-inventario-valorizado"],
    queryFn: apiReportes.inventarioValorizado,
  });

  const valorTotal = (filas ?? []).reduce((acc, f) => acc + f.valorTotal, 0);
  const filasExportar = (filas ?? []).map((f) => [
    f.sku,
    f.nombre,
    f.categoria ?? "",
    String(f.stockActual),
    formatearMoneda(f.costoUnitario),
    formatearMoneda(f.valorTotal),
  ]);

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <p className="text-sm text-muted-foreground">
          Valor total del inventario:{" "}
          <span className="font-medium text-foreground">{formatearMoneda(valorTotal)}</span>
        </p>
        <ExportarReporteBotones
          titulo="Inventario valorizado"
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
              <TableHead>Categoría</TableHead>
              <TableHead className="text-right">Stock</TableHead>
              <TableHead className="text-right">Costo unit.</TableHead>
              <TableHead className="text-right">Valor total</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {isLoading && (
              <TableRow>
                <TableCell colSpan={6} className="text-center text-muted-foreground">
                  Cargando…
                </TableCell>
              </TableRow>
            )}
            {filas?.map((f) => (
              <TableRow key={f.sku}>
                <TableCell className="font-mono text-sm">{f.sku}</TableCell>
                <TableCell>{f.nombre}</TableCell>
                <TableCell className="text-muted-foreground">{f.categoria ?? "—"}</TableCell>
                <TableCell className="text-right">{f.stockActual}</TableCell>
                <TableCell className="text-right">{formatearMoneda(f.costoUnitario)}</TableCell>
                <TableCell className="text-right font-medium">
                  {formatearMoneda(f.valorTotal)}
                </TableCell>
              </TableRow>
            ))}
          </TableBody>
        </Table>
      </div>
    </div>
  );
}

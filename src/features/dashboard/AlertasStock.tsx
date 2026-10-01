import { useQuery } from "@tanstack/react-query";
import { apiInventario } from "@/lib/api-inventario";
import { Badge } from "@/components/ui/badge";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";

export function AlertasStock() {
  const { data, isLoading } = useQuery({
    queryKey: ["productos", { soloStockBajo: true, dashboard: true }],
    queryFn: () => apiInventario.productoList({ soloStockBajo: true, porPagina: 20, pagina: 1 }),
  });

  if (!isLoading && (data?.items.length ?? 0) === 0) return null;

  return (
    <Card>
      <CardHeader>
        <CardTitle className="text-base">Alertas de stock mínimo</CardTitle>
      </CardHeader>
      <CardContent>
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead>SKU</TableHead>
              <TableHead>Producto</TableHead>
              <TableHead className="text-right">Stock actual</TableHead>
              <TableHead className="text-right">Stock mínimo</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {data?.items.map((p) => (
              <TableRow key={p.id}>
                <TableCell className="font-mono text-sm">{p.sku}</TableCell>
                <TableCell>{p.nombre}</TableCell>
                <TableCell className="text-right">
                  <Badge variant={p.stockActual === 0 ? "destructive" : "warning"}>
                    {p.stockActual}
                  </Badge>
                </TableCell>
                <TableCell className="text-right text-muted-foreground">{p.stockMinimo}</TableCell>
              </TableRow>
            ))}
          </TableBody>
        </Table>
      </CardContent>
    </Card>
  );
}

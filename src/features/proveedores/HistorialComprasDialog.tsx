import { useQuery } from "@tanstack/react-query";
import { apiInventario, type Proveedor } from "@/lib/api-inventario";
import { formatearMoneda, formatearFechaHora } from "@/lib/format";
import { Badge } from "@/components/ui/badge";
import { Dialog, DialogContent, DialogHeader, DialogTitle } from "@/components/ui/dialog";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";

interface Props {
  abierto: boolean;
  onOpenChange: (abierto: boolean) => void;
  proveedor: Proveedor | null;
}

export function HistorialComprasDialog({ abierto, onOpenChange, proveedor }: Props) {
  const { data: compras, isLoading } = useQuery({
    queryKey: ["compras", "proveedor", proveedor?.id],
    queryFn: () => apiInventario.proveedorHistorialCompras(proveedor!.id),
    enabled: abierto && !!proveedor,
  });

  return (
    <Dialog open={abierto} onOpenChange={onOpenChange}>
      <DialogContent className="max-h-[90vh] max-w-2xl overflow-y-auto">
        <DialogHeader>
          <DialogTitle>Historial de compras — {proveedor?.razonSocial}</DialogTitle>
        </DialogHeader>

        <Table>
          <TableHeader>
            <TableRow>
              <TableHead>Número</TableHead>
              <TableHead>Fecha</TableHead>
              <TableHead className="text-right">Total</TableHead>
              <TableHead>Estado</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {isLoading && (
              <TableRow>
                <TableCell colSpan={4} className="text-center text-muted-foreground">
                  Cargando…
                </TableCell>
              </TableRow>
            )}
            {!isLoading && compras?.length === 0 && (
              <TableRow>
                <TableCell colSpan={4} className="text-center text-muted-foreground">
                  Este proveedor todavía no tiene compras registradas.
                </TableCell>
              </TableRow>
            )}
            {compras?.map((c) => (
              <TableRow key={c.id}>
                <TableCell className="font-mono text-sm">{c.numero}</TableCell>
                <TableCell className="whitespace-nowrap text-sm">
                  {formatearFechaHora(c.fecha)}
                </TableCell>
                <TableCell className="text-right">{formatearMoneda(c.total)}</TableCell>
                <TableCell>
                  <Badge variant={c.estado === "registrada" ? "success" : "destructive"}>
                    {c.estado}
                  </Badge>
                </TableCell>
              </TableRow>
            ))}
          </TableBody>
        </Table>
      </DialogContent>
    </Dialog>
  );
}

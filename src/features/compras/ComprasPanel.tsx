import { useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { toast } from "sonner";
import { apiInventario, type Compra } from "@/lib/api-inventario";
import { formatearMoneda, formatearFechaHora } from "@/lib/format";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
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
import {
  Dialog,
  DialogContent,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { CompraFormDialog } from "@/features/compras/CompraFormDialog";

export function ComprasPanel() {
  const queryClient = useQueryClient();
  const [dialogoAbierto, setDialogoAbierto] = useState(false);
  const [compraAAnular, setCompraAAnular] = useState<Compra | null>(null);
  const [motivo, setMotivo] = useState("");

  const { data: compras, isLoading } = useQuery({
    queryKey: ["compras"],
    queryFn: () => apiInventario.compraList(null),
  });

  const anularMutacion = useMutation({
    mutationFn: () => apiInventario.compraAnular(compraAAnular!.id, motivo),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["compras"] });
      queryClient.invalidateQueries({ queryKey: ["productos"] });
      toast.success("Compra anulada.");
      setCompraAAnular(null);
      setMotivo("");
    },
    onError: (err) => toast.error(String(err)),
  });

  return (
    <div className="space-y-4">
      <div className="flex justify-end">
        <Button onClick={() => setDialogoAbierto(true)}>Registrar compra</Button>
      </div>

      <div className="rounded-md border border-border">
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead>Número</TableHead>
              <TableHead>Fecha</TableHead>
              <TableHead>Proveedor</TableHead>
              <TableHead className="text-right">Total</TableHead>
              <TableHead>Estado</TableHead>
              <TableHead className="text-right">Acciones</TableHead>
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
            {!isLoading && compras?.length === 0 && (
              <TableRow>
                <TableCell colSpan={6} className="text-center text-muted-foreground">
                  Sin compras registradas.
                </TableCell>
              </TableRow>
            )}
            {compras?.map((c) => (
              <TableRow key={c.id}>
                <TableCell className="font-mono text-sm">{c.numero}</TableCell>
                <TableCell className="whitespace-nowrap text-sm">
                  {formatearFechaHora(c.fecha)}
                </TableCell>
                <TableCell>{c.proveedorNombre ?? "—"}</TableCell>
                <TableCell className="text-right">{formatearMoneda(c.total)}</TableCell>
                <TableCell>
                  <Badge variant={c.estado === "registrada" ? "success" : "destructive"}>
                    {c.estado}
                  </Badge>
                </TableCell>
                <TableCell className="text-right">
                  {c.estado === "registrada" && (
                    <Button variant="destructive" size="sm" onClick={() => setCompraAAnular(c)}>
                      Anular
                    </Button>
                  )}
                </TableCell>
              </TableRow>
            ))}
          </TableBody>
        </Table>
      </div>

      <CompraFormDialog abierto={dialogoAbierto} onOpenChange={setDialogoAbierto} />

      <Dialog open={!!compraAAnular} onOpenChange={(v) => !v && setCompraAAnular(null)}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Anular compra {compraAAnular?.numero}</DialogTitle>
          </DialogHeader>
          <p className="text-sm text-muted-foreground">
            Esta acción revierte el stock que ingresó con esta compra (se registra como salida en el
            kardex). Requiere un motivo.
          </p>
          <div className="space-y-1.5">
            <Label htmlFor="motivoAnulacion">Motivo</Label>
            <Input
              id="motivoAnulacion"
              value={motivo}
              onChange={(e) => setMotivo(e.currentTarget.value)}
            />
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => setCompraAAnular(null)}>
              Cancelar
            </Button>
            <Button
              variant="destructive"
              disabled={!motivo.trim() || anularMutacion.isPending}
              onClick={() => anularMutacion.mutate()}
            >
              Anular compra
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  );
}

import { useState } from "react";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import { toast } from "sonner";
import { apiVentas, type VentaConDetalle } from "@/lib/api-ventas";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import {
  Dialog,
  DialogContent,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
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
  venta: VentaConDetalle | null;
}

export function DevolucionDialog({ abierto, onOpenChange, venta }: Props) {
  const queryClient = useQueryClient();
  const [cantidades, setCantidades] = useState<Record<number, string>>({});
  const [motivo, setMotivo] = useState("");

  const mutacion = useMutation({
    mutationFn: () => {
      const items = Object.entries(cantidades)
        .map(([ventaDetalleId, cantidad]) => ({
          ventaDetalleId: Number(ventaDetalleId),
          cantidad: Number(cantidad),
        }))
        .filter((i) => i.cantidad > 0);
      return apiVentas.devolucionCrear({ ventaId: venta!.venta.id, motivo, items });
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["ventas"] });
      queryClient.invalidateQueries({ queryKey: ["venta", venta?.venta.id] });
      queryClient.invalidateQueries({ queryKey: ["productos"] });
      toast.success("Devolución registrada.");
      setCantidades({});
      setMotivo("");
      onOpenChange(false);
    },
    onError: (err) => toast.error(String(err)),
  });

  const hayAlgo = Object.values(cantidades).some((v) => Number(v) > 0);

  return (
    <Dialog open={abierto} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-xl">
        <DialogHeader>
          <DialogTitle>Devolución — {venta?.venta.numeroComprobante}</DialogTitle>
        </DialogHeader>

        <Table>
          <TableHeader>
            <TableRow>
              <TableHead>Producto</TableHead>
              <TableHead className="text-right">Vendido</TableHead>
              <TableHead className="text-right">Ya devuelto</TableHead>
              <TableHead className="text-right">A devolver</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {venta?.items.map((i) => {
              const restante = i.cantidad - i.cantidadDevuelta;
              return (
                <TableRow key={i.id}>
                  <TableCell>{i.productoNombre}</TableCell>
                  <TableCell className="text-right">{i.cantidad}</TableCell>
                  <TableCell className="text-right">{i.cantidadDevuelta}</TableCell>
                  <TableCell className="text-right">
                    <Input
                      type="number"
                      min="0"
                      max={restante}
                      step="1"
                      className="ml-auto w-20"
                      disabled={restante <= 0}
                      value={cantidades[i.id] ?? ""}
                      onChange={(e) =>
                        setCantidades((prev) => ({ ...prev, [i.id]: e.currentTarget.value }))
                      }
                    />
                  </TableCell>
                </TableRow>
              );
            })}
          </TableBody>
        </Table>

        <div className="space-y-1.5">
          <Label htmlFor="motivoDevolucion">Motivo</Label>
          <Input
            id="motivoDevolucion"
            value={motivo}
            onChange={(e) => setMotivo(e.currentTarget.value)}
          />
        </div>

        <DialogFooter>
          <Button variant="outline" onClick={() => onOpenChange(false)}>
            Cancelar
          </Button>
          <Button
            disabled={!hayAlgo || !motivo.trim() || mutacion.isPending}
            onClick={() => mutacion.mutate()}
          >
            Registrar devolución
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}

import { useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { toast } from "sonner";
import { apiVentas } from "@/lib/api-ventas";
import { formatearMoneda, formatearFechaHora } from "@/lib/format";
import { Badge } from "@/components/ui/badge";
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
import { DevolucionDialog } from "@/features/ventas/DevolucionDialog";
import { ComprobanteDialog } from "@/features/comprobantes/ComprobanteDialog";

interface Props {
  ventaId: number | null;
  onOpenChange: (abierto: boolean) => void;
}

export function VentaDetalleDialog({ ventaId, onOpenChange }: Props) {
  const queryClient = useQueryClient();
  const [motivoAnulacion, setMotivoAnulacion] = useState("");
  const [anularAbierto, setAnularAbierto] = useState(false);
  const [devolucionAbierta, setDevolucionAbierta] = useState(false);
  const [comprobanteAbierto, setComprobanteAbierto] = useState(false);

  const { data: venta } = useQuery({
    queryKey: ["venta", ventaId],
    queryFn: () => apiVentas.ventaGet(ventaId!),
    enabled: ventaId != null,
  });

  const anularMutacion = useMutation({
    mutationFn: () => apiVentas.ventaAnular(ventaId!, motivoAnulacion),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["ventas"] });
      queryClient.invalidateQueries({ queryKey: ["venta", ventaId] });
      queryClient.invalidateQueries({ queryKey: ["productos"] });
      toast.success("Venta anulada.");
      setAnularAbierto(false);
      setMotivoAnulacion("");
    },
    onError: (err) => toast.error(String(err)),
  });

  const puedeAnular =
    venta?.venta.estado === "completada" && venta.items.every((i) => i.cantidadDevuelta === 0);
  const puedeDevolver =
    venta?.venta.estado === "completada" &&
    venta.items.some((i) => i.cantidadDevuelta < i.cantidad);

  return (
    <>
      <Dialog open={ventaId != null} onOpenChange={onOpenChange}>
        <DialogContent className="max-h-[90vh] max-w-2xl overflow-y-auto">
          <DialogHeader>
            <DialogTitle>Venta {venta?.venta.numeroComprobante}</DialogTitle>
          </DialogHeader>

          {venta && (
            <>
              <div className="flex flex-wrap items-center gap-3 text-sm">
                <Badge variant={venta.venta.estado === "completada" ? "success" : "destructive"}>
                  {venta.venta.estado}
                </Badge>
                <span className="text-muted-foreground">
                  {formatearFechaHora(venta.venta.fecha)}
                </span>
                <span className="text-muted-foreground">
                  Cliente: {venta.venta.clienteNombre ?? "General"}
                </span>
                <span className="text-muted-foreground">Pago: {venta.venta.metodoPago}</span>
              </div>

              {venta.venta.motivoAnulacion && (
                <p className="text-sm text-destructive">
                  Motivo de anulación: {venta.venta.motivoAnulacion}
                </p>
              )}

              <Table>
                <TableHeader>
                  <TableRow>
                    <TableHead>Producto</TableHead>
                    <TableHead className="text-right">Cant.</TableHead>
                    <TableHead className="text-right">Precio</TableHead>
                    <TableHead className="text-right">Devuelto</TableHead>
                    <TableHead className="text-right">Subtotal</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {venta.items.map((i) => (
                    <TableRow key={i.id}>
                      <TableCell>{i.productoNombre}</TableCell>
                      <TableCell className="text-right">{i.cantidad}</TableCell>
                      <TableCell className="text-right">
                        {formatearMoneda(i.precioUnitario)}
                      </TableCell>
                      <TableCell className="text-right">{i.cantidadDevuelta || "—"}</TableCell>
                      <TableCell className="text-right">{formatearMoneda(i.subtotal)}</TableCell>
                    </TableRow>
                  ))}
                </TableBody>
              </Table>

              <div className="ml-auto max-w-xs space-y-1 text-sm">
                <div className="flex justify-between">
                  <span className="text-muted-foreground">Subtotal</span>
                  <span>{formatearMoneda(venta.venta.subtotal)}</span>
                </div>
                <div className="flex justify-between">
                  <span className="text-muted-foreground">Descuentos</span>
                  <span>-{formatearMoneda(venta.venta.descuentoTotal)}</span>
                </div>
                <div className="flex justify-between">
                  <span className="text-muted-foreground">Impuestos</span>
                  <span>{formatearMoneda(venta.venta.impuestos)}</span>
                </div>
                <div className="flex justify-between font-semibold">
                  <span>Total</span>
                  <span>{formatearMoneda(venta.venta.total)}</span>
                </div>
                {venta.venta.valorRecibido != null && (
                  <>
                    <div className="flex justify-between">
                      <span className="text-muted-foreground">Recibido</span>
                      <span>{formatearMoneda(venta.venta.valorRecibido)}</span>
                    </div>
                    <div className="flex justify-between">
                      <span className="text-muted-foreground">Cambio</span>
                      <span>{formatearMoneda(venta.venta.cambio ?? 0)}</span>
                    </div>
                  </>
                )}
              </div>

              <DialogFooter>
                <Button variant="outline" onClick={() => setComprobanteAbierto(true)}>
                  Comprobante
                </Button>
                {puedeDevolver && (
                  <Button variant="outline" onClick={() => setDevolucionAbierta(true)}>
                    Registrar devolución
                  </Button>
                )}
                {puedeAnular && (
                  <Button variant="destructive" onClick={() => setAnularAbierto(true)}>
                    Anular venta
                  </Button>
                )}
              </DialogFooter>
            </>
          )}
        </DialogContent>
      </Dialog>

      <Dialog open={anularAbierto} onOpenChange={setAnularAbierto}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Anular venta {venta?.venta.numeroComprobante}</DialogTitle>
          </DialogHeader>
          <p className="text-sm text-muted-foreground">
            Se revierte el stock de todos los productos de esta venta. Requiere un motivo.
          </p>
          <div className="space-y-1.5">
            <Label htmlFor="motivoAnulacionVenta">Motivo</Label>
            <Input
              id="motivoAnulacionVenta"
              value={motivoAnulacion}
              onChange={(e) => setMotivoAnulacion(e.currentTarget.value)}
            />
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => setAnularAbierto(false)}>
              Cancelar
            </Button>
            <Button
              variant="destructive"
              disabled={!motivoAnulacion.trim() || anularMutacion.isPending}
              onClick={() => anularMutacion.mutate()}
            >
              Anular
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      <DevolucionDialog
        abierto={devolucionAbierta}
        onOpenChange={setDevolucionAbierta}
        venta={venta ?? null}
      />

      <ComprobanteDialog
        ventaId={comprobanteAbierto ? (ventaId ?? null) : null}
        onOpenChange={setComprobanteAbierto}
        modo="regenerar"
      />
    </>
  );
}

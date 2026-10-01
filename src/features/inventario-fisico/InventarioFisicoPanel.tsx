import { useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { toast } from "sonner";
import { apiInventario, type InventarioFisico } from "@/lib/api-inventario";
import { formatearFechaHora } from "@/lib/format";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Input } from "@/components/ui/input";
import { Textarea } from "@/components/ui/textarea";
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
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from "@/components/ui/alert-dialog";

function FilaConteo({
  inventarioFisicoId,
  productoId,
  sku,
  nombre,
  stockActual,
  stockContado,
  diferencia,
  cerrado,
}: {
  inventarioFisicoId: number;
  productoId: number;
  sku: string;
  nombre: string;
  stockActual: number;
  stockContado: number | null;
  diferencia: number | null;
  cerrado: boolean;
}) {
  const queryClient = useQueryClient();
  const [valor, setValor] = useState(stockContado != null ? String(stockContado) : "");

  const mutacion = useMutation({
    mutationFn: () =>
      apiInventario.inventarioFisicoRegistrarConteo({
        inventarioFisicoId,
        productoId,
        stockContado: Number(valor),
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["inventario-fisico", inventarioFisicoId] });
    },
    onError: (err) => toast.error(String(err)),
  });

  return (
    <TableRow>
      <TableCell className="font-mono text-sm">{sku}</TableCell>
      <TableCell>{nombre}</TableCell>
      <TableCell className="text-right">{stockActual}</TableCell>
      <TableCell className="text-right">
        <Input
          type="number"
          step="1"
          min="0"
          className="ml-auto w-24"
          value={valor}
          disabled={cerrado}
          onChange={(e) => setValor(e.currentTarget.value)}
        />
      </TableCell>
      <TableCell className="text-right">{diferencia ?? "—"}</TableCell>
      <TableCell className="text-right">
        {!cerrado && (
          <Button
            size="sm"
            variant="outline"
            disabled={!valor || mutacion.isPending}
            onClick={() => mutacion.mutate()}
          >
            Guardar
          </Button>
        )}
      </TableCell>
    </TableRow>
  );
}

function DetalleInventarioFisico({ id, onClose }: { id: number; onClose: () => void }) {
  const queryClient = useQueryClient();
  const [confirmarCierre, setConfirmarCierre] = useState(false);

  const { data } = useQuery({
    queryKey: ["inventario-fisico", id],
    queryFn: () => apiInventario.inventarioFisicoGet(id),
  });
  const { data: catalogo } = useQuery({
    queryKey: ["productos", "catalogo-completo"],
    queryFn: () => apiInventario.productoList({ porPagina: 500, pagina: 1 }),
  });

  const cerrarMutacion = useMutation({
    mutationFn: () => apiInventario.inventarioFisicoCerrar(id),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["inventario-fisico", id] });
      queryClient.invalidateQueries({ queryKey: ["inventario-fisico-list"] });
      queryClient.invalidateQueries({ queryKey: ["productos"] });
      toast.success("Toma de inventario cerrada. Se aplicaron los ajustes de stock.");
      setConfirmarCierre(false);
    },
    onError: (err) => toast.error(String(err)),
  });

  if (!data || !catalogo) return <p className="text-muted-foreground">Cargando…</p>;

  const cerrado = data.cabecera.estado === "cerrado";
  const detallePorProducto = new Map(data.detalle.map((d) => [d.productoId, d]));

  return (
    <>
      <div className="max-h-96 overflow-y-auto rounded-md border border-border">
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead>SKU</TableHead>
              <TableHead>Producto</TableHead>
              <TableHead className="text-right">Stock sistema</TableHead>
              <TableHead className="text-right">Contado</TableHead>
              <TableHead className="text-right">Diferencia</TableHead>
              <TableHead className="text-right"></TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {catalogo.items.map((p) => {
              const existente = detallePorProducto.get(p.id);
              return (
                <FilaConteo
                  key={p.id}
                  inventarioFisicoId={id}
                  productoId={p.id}
                  sku={p.sku}
                  nombre={p.nombre}
                  stockActual={existente?.stockSistema ?? p.stockActual}
                  stockContado={existente?.stockContado ?? null}
                  diferencia={existente?.diferencia ?? null}
                  cerrado={cerrado}
                />
              );
            })}
          </TableBody>
        </Table>
      </div>

      <DialogFooter>
        <Button variant="outline" onClick={onClose}>
          Cerrar ventana
        </Button>
        {!cerrado && (
          <Button variant="destructive" onClick={() => setConfirmarCierre(true)}>
            Cerrar toma y ajustar stock
          </Button>
        )}
      </DialogFooter>

      <AlertDialog open={confirmarCierre} onOpenChange={setConfirmarCierre}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>¿Cerrar esta toma de inventario?</AlertDialogTitle>
            <AlertDialogDescription>
              Se creará un movimiento de ajuste por cada producto con diferencia distinta de cero.
              Esta acción no se puede deshacer.
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>Cancelar</AlertDialogCancel>
            <AlertDialogAction onClick={() => cerrarMutacion.mutate()}>
              Cerrar y ajustar
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </>
  );
}

export function InventarioFisicoPanel() {
  const queryClient = useQueryClient();
  const [detalleAbierto, setDetalleAbierto] = useState<number | null>(null);
  const [nuevaAbierta, setNuevaAbierta] = useState(false);
  const [observaciones, setObservaciones] = useState("");

  const { data: tomas, isLoading } = useQuery({
    queryKey: ["inventario-fisico-list"],
    queryFn: apiInventario.inventarioFisicoList,
  });

  const iniciarMutacion = useMutation({
    mutationFn: () => apiInventario.inventarioFisicoIniciar(observaciones.trim() || null),
    onSuccess: (creada: InventarioFisico) => {
      queryClient.invalidateQueries({ queryKey: ["inventario-fisico-list"] });
      setNuevaAbierta(false);
      setObservaciones("");
      setDetalleAbierto(creada.id);
    },
    onError: (err) => toast.error(String(err)),
  });

  return (
    <div className="space-y-4">
      <div className="flex justify-end">
        <Button onClick={() => setNuevaAbierta(true)}>Nueva toma de inventario</Button>
      </div>

      <div className="rounded-md border border-border">
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead>Fecha</TableHead>
              <TableHead>Estado</TableHead>
              <TableHead>Observaciones</TableHead>
              <TableHead className="text-right">Acciones</TableHead>
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
            {!isLoading && tomas?.length === 0 && (
              <TableRow>
                <TableCell colSpan={4} className="text-center text-muted-foreground">
                  Sin tomas de inventario registradas.
                </TableCell>
              </TableRow>
            )}
            {tomas?.map((t) => (
              <TableRow key={t.id}>
                <TableCell className="whitespace-nowrap text-sm">
                  {formatearFechaHora(t.fecha)}
                </TableCell>
                <TableCell>
                  <Badge variant={t.estado === "abierto" ? "warning" : "success"}>{t.estado}</Badge>
                </TableCell>
                <TableCell className="text-muted-foreground">{t.observaciones ?? "—"}</TableCell>
                <TableCell className="text-right">
                  <Button variant="outline" size="sm" onClick={() => setDetalleAbierto(t.id)}>
                    Ver / contar
                  </Button>
                </TableCell>
              </TableRow>
            ))}
          </TableBody>
        </Table>
      </div>

      <Dialog open={nuevaAbierta} onOpenChange={setNuevaAbierta}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Nueva toma de inventario</DialogTitle>
          </DialogHeader>
          <div className="space-y-1.5">
            <Label htmlFor="obsInventario">Observaciones (opcional)</Label>
            <Textarea
              id="obsInventario"
              value={observaciones}
              onChange={(e) => setObservaciones(e.currentTarget.value)}
            />
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => setNuevaAbierta(false)}>
              Cancelar
            </Button>
            <Button onClick={() => iniciarMutacion.mutate()} disabled={iniciarMutacion.isPending}>
              Iniciar
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      <Dialog open={detalleAbierto != null} onOpenChange={(v) => !v && setDetalleAbierto(null)}>
        <DialogContent className="max-h-[90vh] max-w-3xl overflow-y-auto">
          <DialogHeader>
            <DialogTitle>Toma de inventario #{detalleAbierto}</DialogTitle>
          </DialogHeader>
          {detalleAbierto != null && (
            <DetalleInventarioFisico id={detalleAbierto} onClose={() => setDetalleAbierto(null)} />
          )}
        </DialogContent>
      </Dialog>
    </div>
  );
}

import { useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { toast } from "sonner";
import { apiVentas } from "@/lib/api-ventas";
import { formatearMoneda } from "@/lib/format";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Textarea } from "@/components/ui/textarea";
import {
  Dialog,
  DialogContent,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";

export function CajaWidget() {
  const queryClient = useQueryClient();
  const [abrirDialogo, setAbrirDialogo] = useState(false);
  const [cerrarDialogo, setCerrarDialogo] = useState(false);
  const [montoApertura, setMontoApertura] = useState("");
  const [montoContado, setMontoContado] = useState("");
  const [observaciones, setObservaciones] = useState("");

  const { data: sesion, isLoading } = useQuery({
    queryKey: ["caja-actual"],
    queryFn: apiVentas.cajaEstadoActual,
  });

  const abrirMutacion = useMutation({
    mutationFn: () => apiVentas.cajaAbrir(Math.round(Number(montoApertura || "0") * 100)),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["caja-actual"] });
      toast.success("Caja abierta.");
      setAbrirDialogo(false);
      setMontoApertura("");
    },
    onError: (err) => toast.error(String(err)),
  });

  const cerrarMutacion = useMutation({
    mutationFn: () =>
      apiVentas.cajaCerrar(
        Math.round(Number(montoContado || "0") * 100),
        observaciones.trim() || null,
      ),
    onSuccess: (cerrada) => {
      queryClient.invalidateQueries({ queryKey: ["caja-actual"] });
      const diferencia = cerrada.diferencia ?? 0;
      if (diferencia === 0) toast.success("Caja cerrada. Cuadró exacto.");
      else toast.warning(`Caja cerrada. Diferencia: ${formatearMoneda(diferencia)}`);
      setCerrarDialogo(false);
      setMontoContado("");
      setObservaciones("");
    },
    onError: (err) => toast.error(String(err)),
  });

  if (isLoading) return null;

  return (
    <div className="flex items-center gap-3 rounded-md border border-border bg-card px-3 py-2">
      {sesion ? (
        <>
          <Badge variant="success">Caja abierta</Badge>
          <span className="text-sm text-muted-foreground">
            Apertura: {formatearMoneda(sesion.montoApertura)}
          </span>
          <div className="flex-1" />
          <Button size="sm" variant="outline" onClick={() => setCerrarDialogo(true)}>
            Cerrar caja
          </Button>
        </>
      ) : (
        <>
          <Badge variant="secondary">Caja cerrada</Badge>
          <div className="flex-1" />
          <Button size="sm" onClick={() => setAbrirDialogo(true)}>
            Abrir caja
          </Button>
        </>
      )}

      <Dialog open={abrirDialogo} onOpenChange={setAbrirDialogo}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Abrir caja</DialogTitle>
          </DialogHeader>
          <div className="space-y-1.5">
            <Label htmlFor="montoApertura">Monto de apertura</Label>
            <Input
              id="montoApertura"
              type="number"
              min="0"
              step="0.01"
              value={montoApertura}
              onChange={(e) => setMontoApertura(e.currentTarget.value)}
            />
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => setAbrirDialogo(false)}>
              Cancelar
            </Button>
            <Button onClick={() => abrirMutacion.mutate()} disabled={abrirMutacion.isPending}>
              Abrir
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      <Dialog open={cerrarDialogo} onOpenChange={setCerrarDialogo}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Cerrar caja</DialogTitle>
          </DialogHeader>
          <div className="space-y-1.5">
            <Label htmlFor="montoContado">Monto contado en caja</Label>
            <Input
              id="montoContado"
              type="number"
              min="0"
              step="0.01"
              value={montoContado}
              onChange={(e) => setMontoContado(e.currentTarget.value)}
            />
          </div>
          <div className="space-y-1.5">
            <Label htmlFor="obsCierre">Observaciones (opcional)</Label>
            <Textarea
              id="obsCierre"
              value={observaciones}
              onChange={(e) => setObservaciones(e.currentTarget.value)}
            />
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => setCerrarDialogo(false)}>
              Cancelar
            </Button>
            <Button onClick={() => cerrarMutacion.mutate()} disabled={cerrarMutacion.isPending}>
              Cerrar caja
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  );
}

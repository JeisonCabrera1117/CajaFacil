import { useEffect, useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { toast } from "sonner";
import { apiInventario, type Proveedor } from "@/lib/api-inventario";
import { Button } from "@/components/ui/button";
import { Checkbox } from "@/components/ui/checkbox";
import {
  Dialog,
  DialogContent,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";

interface Props {
  abierto: boolean;
  onOpenChange: (abierto: boolean) => void;
  proveedor: Proveedor | null;
}

export function ProductosQueSuministraDialog({ abierto, onOpenChange, proveedor }: Props) {
  const queryClient = useQueryClient();
  const [seleccionados, setSeleccionados] = useState<Set<number>>(new Set());

  const { data: catalogo } = useQuery({
    queryKey: ["productos", "catalogo-completo"],
    queryFn: () => apiInventario.productoList({ porPagina: 500, pagina: 1 }),
    enabled: abierto,
  });
  const { data: asignados } = useQuery({
    queryKey: ["proveedor-productos", proveedor?.id],
    queryFn: () => apiInventario.proveedorProductosList(proveedor!.id),
    enabled: abierto && !!proveedor,
  });

  useEffect(() => {
    // Sincroniza el checklist editable con los datos ya asignados que llegan de forma
    // asíncrona desde el backend; no hay forma de derivarlo en render porque es estado
    // editable independiente (el usuario puede des/marcar antes de guardar).
    // eslint-disable-next-line react-hooks/set-state-in-effect
    if (asignados) setSeleccionados(new Set(asignados));
  }, [asignados]);

  const mutacion = useMutation({
    mutationFn: () =>
      apiInventario.proveedorProductosAsignar(proveedor!.id, Array.from(seleccionados)),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["proveedor-productos", proveedor?.id] });
      toast.success("Productos actualizados.");
      onOpenChange(false);
    },
    onError: (err) => toast.error(String(err)),
  });

  function alternar(id: number) {
    setSeleccionados((prev) => {
      const copia = new Set(prev);
      if (copia.has(id)) copia.delete(id);
      else copia.add(id);
      return copia;
    });
  }

  return (
    <Dialog open={abierto} onOpenChange={onOpenChange}>
      <DialogContent className="max-h-[90vh] max-w-lg overflow-y-auto">
        <DialogHeader>
          <DialogTitle>Productos que suministra — {proveedor?.razonSocial}</DialogTitle>
        </DialogHeader>

        <div className="space-y-2">
          {catalogo?.items.map((p) => (
            <label
              key={p.id}
              className="flex items-center gap-2 rounded-md border border-border p-2 text-sm"
            >
              <Checkbox checked={seleccionados.has(p.id)} onCheckedChange={() => alternar(p.id)} />
              <span className="font-mono text-xs text-muted-foreground">{p.sku}</span>
              <span>{p.nombre}</span>
            </label>
          ))}
          {catalogo?.items.length === 0 && (
            <p className="text-sm text-muted-foreground">No hay productos creados todavía.</p>
          )}
        </div>

        <DialogFooter>
          <Button variant="outline" onClick={() => onOpenChange(false)}>
            Cancelar
          </Button>
          <Button onClick={() => mutacion.mutate()} disabled={mutacion.isPending}>
            Guardar
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}

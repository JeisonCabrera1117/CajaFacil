import { useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { toast } from "sonner";
import { Plus, Trash2 } from "lucide-react";
import { apiInventario } from "@/lib/api-inventario";
import {
  Dialog,
  DialogContent,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Textarea } from "@/components/ui/textarea";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";

interface ItemForm {
  productoId: string;
  cantidad: string;
  costoUnitario: string;
}

const ITEM_VACIO: ItemForm = { productoId: "", cantidad: "", costoUnitario: "" };

interface Props {
  abierto: boolean;
  onOpenChange: (abierto: boolean) => void;
}

export function CompraFormDialog({ abierto, onOpenChange }: Props) {
  const queryClient = useQueryClient();
  const { data: proveedores } = useQuery({
    queryKey: ["proveedores"],
    queryFn: apiInventario.proveedorList,
  });
  const { data: catalogoProductos } = useQuery({
    queryKey: ["productos", "catalogo-completo"],
    queryFn: () => apiInventario.productoList({ porPagina: 500, pagina: 1 }),
    enabled: abierto,
  });

  const [proveedorId, setProveedorId] = useState("none");
  const [numero, setNumero] = useState("");
  const [observaciones, setObservaciones] = useState("");
  const [items, setItems] = useState<ItemForm[]>([{ ...ITEM_VACIO }]);

  function limpiar() {
    setProveedorId("none");
    setNumero("");
    setObservaciones("");
    setItems([{ ...ITEM_VACIO }]);
  }

  const mutacion = useMutation({
    mutationFn: () =>
      apiInventario.compraCreate({
        proveedorId: proveedorId !== "none" ? Number(proveedorId) : null,
        numero: numero.trim(),
        observaciones: observaciones.trim() || null,
        items: items
          .filter((i) => i.productoId && i.cantidad)
          .map((i) => ({
            productoId: Number(i.productoId),
            cantidad: Number(i.cantidad),
            costoUnitario: Math.round(Number(i.costoUnitario || "0") * 100),
          })),
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["compras"] });
      queryClient.invalidateQueries({ queryKey: ["productos"] });
      toast.success("Compra registrada.");
      limpiar();
      onOpenChange(false);
    },
    onError: (err) => toast.error(String(err)),
  });

  const total = items.reduce(
    (acc, i) => acc + Number(i.cantidad || 0) * Number(i.costoUnitario || 0),
    0,
  );

  return (
    <Dialog
      open={abierto}
      onOpenChange={(v) => {
        if (!v) limpiar();
        onOpenChange(v);
      }}
    >
      <DialogContent className="max-h-[90vh] max-w-3xl overflow-y-auto">
        <DialogHeader>
          <DialogTitle>Registrar compra</DialogTitle>
        </DialogHeader>

        <div className="space-y-4">
          <div className="grid grid-cols-2 gap-4">
            <div className="space-y-1.5">
              <Label>Proveedor (opcional)</Label>
              <Select value={proveedorId} onValueChange={(v) => setProveedorId(v ?? "none")}>
                <SelectTrigger>
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="none">Sin proveedor</SelectItem>
                  {proveedores?.map((p) => (
                    <SelectItem key={p.id} value={String(p.id)}>
                      {p.razonSocial}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>
            <div className="space-y-1.5">
              <Label htmlFor="numero">Número de compra</Label>
              <Input
                id="numero"
                value={numero}
                onChange={(e) => setNumero(e.currentTarget.value)}
              />
            </div>
          </div>

          <div className="space-y-1.5">
            <Label htmlFor="observaciones">Observaciones</Label>
            <Textarea
              id="observaciones"
              value={observaciones}
              onChange={(e) => setObservaciones(e.currentTarget.value)}
            />
          </div>

          <div className="space-y-2">
            <div className="flex items-center justify-between">
              <Label>Productos</Label>
              <Button
                type="button"
                variant="outline"
                size="sm"
                onClick={() => setItems((prev) => [...prev, { ...ITEM_VACIO }])}
              >
                <Plus className="size-4" /> Agregar ítem
              </Button>
            </div>

            {items.map((item, idx) => (
              <div key={idx} className="grid grid-cols-[1fr_100px_120px_32px] items-end gap-2">
                <div className="space-y-1.5">
                  {idx === 0 && <Label className="text-xs">Producto</Label>}
                  <Select
                    value={item.productoId}
                    onValueChange={(v) =>
                      setItems((prev) =>
                        prev.map((it, i) => (i === idx ? { ...it, productoId: v ?? "" } : it)),
                      )
                    }
                  >
                    <SelectTrigger>
                      <SelectValue placeholder="Seleccionar…" />
                    </SelectTrigger>
                    <SelectContent>
                      {catalogoProductos?.items.map((p) => (
                        <SelectItem key={p.id} value={String(p.id)}>
                          {p.sku} — {p.nombre}
                        </SelectItem>
                      ))}
                    </SelectContent>
                  </Select>
                </div>
                <div className="space-y-1.5">
                  {idx === 0 && <Label className="text-xs">Cantidad</Label>}
                  <Input
                    type="number"
                    min="1"
                    step="1"
                    value={item.cantidad}
                    onChange={(e) => {
                      const cantidad = e.currentTarget.value;
                      setItems((prev) =>
                        prev.map((it, i) => (i === idx ? { ...it, cantidad } : it)),
                      );
                    }}
                  />
                </div>
                <div className="space-y-1.5">
                  {idx === 0 && <Label className="text-xs">Costo unit.</Label>}
                  <Input
                    type="number"
                    min="0"
                    step="0.01"
                    value={item.costoUnitario}
                    onChange={(e) => {
                      const costoUnitario = e.currentTarget.value;
                      setItems((prev) =>
                        prev.map((it, i) => (i === idx ? { ...it, costoUnitario } : it)),
                      );
                    }}
                  />
                </div>
                <Button
                  type="button"
                  variant="ghost"
                  size="icon-sm"
                  onClick={() => setItems((prev) => prev.filter((_, i) => i !== idx))}
                  disabled={items.length === 1}
                >
                  <Trash2 className="size-4" />
                </Button>
              </div>
            ))}
          </div>

          <p className="text-right text-sm text-muted-foreground">
            Total: {(total / 100).toLocaleString("es-CO", { style: "currency", currency: "COP" })}
          </p>
        </div>

        <DialogFooter>
          <Button type="button" variant="outline" onClick={() => onOpenChange(false)}>
            Cancelar
          </Button>
          <Button
            onClick={() => mutacion.mutate()}
            disabled={mutacion.isPending || !numero.trim() || items.every((i) => !i.productoId)}
          >
            Registrar compra
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}

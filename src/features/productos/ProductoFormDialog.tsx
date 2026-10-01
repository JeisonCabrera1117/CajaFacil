import { useEffect } from "react";
import { useForm, useWatch } from "react-hook-form";
import { zodResolver } from "@hookform/resolvers/zod";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { toast } from "sonner";
import { apiInventario, type Producto } from "@/lib/api-inventario";
import {
  productoSchema,
  valoresPorDefecto,
  type ProductoFormValues,
} from "@/features/productos/schema";
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
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";

function aFormulario(p: Producto): ProductoFormValues {
  return {
    sku: p.sku,
    codigoBarras: p.codigoBarras ?? "",
    nombre: p.nombre,
    descripcion: p.descripcion ?? "",
    categoriaId: p.categoriaId ? String(p.categoriaId) : "none",
    unidadMedida: p.unidadMedida,
    precioCosto: String(p.precioCosto / 100),
    precioVenta: String(p.precioVenta / 100),
    impuestoPct: String(p.impuestoPct),
    stockMinimo: String(p.stockMinimo),
    stockMaximo: p.stockMaximo != null ? String(p.stockMaximo) : "",
    ubicacion: p.ubicacion ?? "",
    proveedorPrincipalId: p.proveedorPrincipalId ? String(p.proveedorPrincipalId) : "none",
    estado: p.estado === "inactivo" ? "inactivo" : "activo",
    imagenPath: p.imagenPath ?? "",
  };
}

interface Props {
  abierto: boolean;
  onOpenChange: (abierto: boolean) => void;
  producto: Producto | null;
}

export function ProductoFormDialog({ abierto, onOpenChange, producto }: Props) {
  const queryClient = useQueryClient();
  const { data: categorias } = useQuery({
    queryKey: ["categorias"],
    queryFn: apiInventario.categoriaList,
  });
  const { data: proveedores } = useQuery({
    queryKey: ["proveedores"],
    queryFn: apiInventario.proveedorList,
  });

  const form = useForm<ProductoFormValues>({
    resolver: zodResolver(productoSchema),
    defaultValues: valoresPorDefecto,
  });

  useEffect(() => {
    if (abierto) {
      form.reset(producto ? aFormulario(producto) : valoresPorDefecto);
    }
  }, [abierto, producto, form]);

  const categoriaId = useWatch({ control: form.control, name: "categoriaId" });
  const proveedorPrincipalId = useWatch({ control: form.control, name: "proveedorPrincipalId" });
  const estado = useWatch({ control: form.control, name: "estado" });

  const mutacion = useMutation({
    mutationFn: (values: ProductoFormValues) => {
      const datos = {
        sku: values.sku.trim(),
        codigoBarras: values.codigoBarras.trim() || null,
        nombre: values.nombre.trim(),
        descripcion: values.descripcion.trim() || null,
        categoriaId:
          values.categoriaId && values.categoriaId !== "none" ? Number(values.categoriaId) : null,
        unidadMedida: values.unidadMedida.trim(),
        precioCosto: Math.round(Number(values.precioCosto) * 100),
        precioVenta: Math.round(Number(values.precioVenta) * 100),
        impuestoPct: Number(values.impuestoPct),
        stockMinimo: Number(values.stockMinimo),
        stockMaximo: values.stockMaximo ? Number(values.stockMaximo) : null,
        ubicacion: values.ubicacion.trim() || null,
        proveedorPrincipalId:
          values.proveedorPrincipalId && values.proveedorPrincipalId !== "none"
            ? Number(values.proveedorPrincipalId)
            : null,
        estado: values.estado,
        imagenPath: values.imagenPath.trim() || null,
      };
      return producto
        ? apiInventario.productoUpdate(producto.id, datos)
        : apiInventario.productoCreate(datos);
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["productos"] });
      toast.success(producto ? "Producto actualizado." : "Producto creado.");
      onOpenChange(false);
    },
    onError: (err) => toast.error(String(err)),
  });

  return (
    <Dialog open={abierto} onOpenChange={onOpenChange}>
      <DialogContent className="max-h-[90vh] max-w-2xl overflow-y-auto">
        <DialogHeader>
          <DialogTitle>{producto ? "Editar producto" : "Nuevo producto"}</DialogTitle>
        </DialogHeader>

        <form onSubmit={form.handleSubmit((v) => mutacion.mutate(v))} className="space-y-4">
          <div className="grid grid-cols-2 gap-4">
            <div className="space-y-1.5">
              <Label htmlFor="sku">SKU</Label>
              <Input id="sku" {...form.register("sku")} />
              {form.formState.errors.sku && (
                <p className="text-sm text-destructive">{form.formState.errors.sku.message}</p>
              )}
            </div>
            <div className="space-y-1.5">
              <Label htmlFor="codigoBarras">Código de barras</Label>
              <Input id="codigoBarras" {...form.register("codigoBarras")} />
            </div>
            <div className="col-span-2 space-y-1.5">
              <Label htmlFor="nombre">Nombre</Label>
              <Input id="nombre" {...form.register("nombre")} />
              {form.formState.errors.nombre && (
                <p className="text-sm text-destructive">{form.formState.errors.nombre.message}</p>
              )}
            </div>
            <div className="col-span-2 space-y-1.5">
              <Label htmlFor="descripcion">Descripción</Label>
              <Input id="descripcion" {...form.register("descripcion")} />
            </div>

            <div className="space-y-1.5">
              <Label>Categoría</Label>
              <Select
                value={categoriaId}
                onValueChange={(v) => form.setValue("categoriaId", v ?? "none")}
              >
                <SelectTrigger>
                  <SelectValue placeholder="Sin categoría" />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="none">Sin categoría</SelectItem>
                  {categorias?.map((c) => (
                    <SelectItem key={c.id} value={String(c.id)}>
                      {c.nombre}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>
            <div className="space-y-1.5">
              <Label>Proveedor principal</Label>
              <Select
                value={proveedorPrincipalId}
                onValueChange={(v) => form.setValue("proveedorPrincipalId", v ?? "none")}
              >
                <SelectTrigger>
                  <SelectValue placeholder="Sin proveedor" />
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
              <Label htmlFor="unidadMedida">Unidad de medida</Label>
              <Input id="unidadMedida" {...form.register("unidadMedida")} />
            </div>
            <div className="space-y-1.5">
              <Label htmlFor="ubicacion">Ubicación</Label>
              <Input id="ubicacion" {...form.register("ubicacion")} />
            </div>

            <div className="space-y-1.5">
              <Label htmlFor="precioCosto">Precio de costo</Label>
              <Input
                id="precioCosto"
                type="number"
                step="0.01"
                min="0"
                {...form.register("precioCosto")}
              />
            </div>
            <div className="space-y-1.5">
              <Label htmlFor="precioVenta">Precio de venta</Label>
              <Input
                id="precioVenta"
                type="number"
                step="0.01"
                min="0"
                {...form.register("precioVenta")}
              />
            </div>
            <div className="space-y-1.5">
              <Label htmlFor="impuestoPct">Impuesto (%)</Label>
              <Input
                id="impuestoPct"
                type="number"
                step="0.01"
                min="0"
                max="100"
                {...form.register("impuestoPct")}
              />
            </div>
            <div className="space-y-1.5">
              <Label>Estado</Label>
              <Select
                value={estado}
                onValueChange={(v) =>
                  form.setValue("estado", (v as "activo" | "inactivo") ?? "activo")
                }
              >
                <SelectTrigger>
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="activo">Activo</SelectItem>
                  <SelectItem value="inactivo">Inactivo</SelectItem>
                </SelectContent>
              </Select>
            </div>

            <div className="space-y-1.5">
              <Label htmlFor="stockMinimo">Stock mínimo</Label>
              <Input
                id="stockMinimo"
                type="number"
                step="1"
                min="0"
                {...form.register("stockMinimo")}
              />
            </div>
            <div className="space-y-1.5">
              <Label htmlFor="stockMaximo">Stock máximo (opcional)</Label>
              <Input
                id="stockMaximo"
                type="number"
                step="1"
                min="0"
                {...form.register("stockMaximo")}
              />
            </div>
          </div>

          {!producto && (
            <p className="text-sm text-muted-foreground">
              El stock inicial se registra después de crear el producto, desde "Ajustar stock"
              (movimiento de tipo entrada).
            </p>
          )}

          <DialogFooter>
            <Button type="button" variant="outline" onClick={() => onOpenChange(false)}>
              Cancelar
            </Button>
            <Button type="submit" disabled={mutacion.isPending}>
              {producto ? "Guardar cambios" : "Crear producto"}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  );
}

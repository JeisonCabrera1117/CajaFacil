import { useEffect } from "react";
import { useForm, useWatch } from "react-hook-form";
import { zodResolver } from "@hookform/resolvers/zod";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import { toast } from "sonner";
import { apiInventario, type Proveedor } from "@/lib/api-inventario";
import {
  proveedorSchema,
  valoresPorDefecto,
  type ProveedorFormValues,
} from "@/features/proveedores/schema";
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
import { Switch } from "@/components/ui/switch";

function aFormulario(p: Proveedor): ProveedorFormValues {
  return {
    nit: p.nit ?? "",
    razonSocial: p.razonSocial,
    contacto: p.contacto ?? "",
    telefono: p.telefono ?? "",
    correo: p.correo ?? "",
    direccion: p.direccion ?? "",
    condicionesPago: p.condicionesPago ?? "",
    activo: p.activo,
  };
}

interface Props {
  abierto: boolean;
  onOpenChange: (abierto: boolean) => void;
  proveedor: Proveedor | null;
}

export function ProveedorFormDialog({ abierto, onOpenChange, proveedor }: Props) {
  const queryClient = useQueryClient();

  const form = useForm<ProveedorFormValues>({
    resolver: zodResolver(proveedorSchema),
    defaultValues: valoresPorDefecto,
  });

  useEffect(() => {
    if (abierto) {
      form.reset(proveedor ? aFormulario(proveedor) : valoresPorDefecto);
    }
  }, [abierto, proveedor, form]);

  const activo = useWatch({ control: form.control, name: "activo" });

  const mutacion = useMutation({
    mutationFn: (values: ProveedorFormValues) => {
      const datos = {
        nit: values.nit.trim() || null,
        razonSocial: values.razonSocial.trim(),
        contacto: values.contacto.trim() || null,
        telefono: values.telefono.trim() || null,
        correo: values.correo.trim() || null,
        direccion: values.direccion.trim() || null,
        condicionesPago: values.condicionesPago.trim() || null,
        activo: values.activo,
      };
      return proveedor
        ? apiInventario.proveedorUpdate(proveedor.id, datos)
        : apiInventario.proveedorCreate(datos);
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["proveedores"] });
      toast.success(proveedor ? "Proveedor actualizado." : "Proveedor creado.");
      onOpenChange(false);
    },
    onError: (err) => toast.error(String(err)),
  });

  return (
    <Dialog open={abierto} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-xl">
        <DialogHeader>
          <DialogTitle>{proveedor ? "Editar proveedor" : "Nuevo proveedor"}</DialogTitle>
        </DialogHeader>

        <form onSubmit={form.handleSubmit((v) => mutacion.mutate(v))} className="space-y-4">
          <div className="grid grid-cols-2 gap-4">
            <div className="col-span-2 space-y-1.5">
              <Label htmlFor="razonSocial">Razón social</Label>
              <Input id="razonSocial" {...form.register("razonSocial")} />
              {form.formState.errors.razonSocial && (
                <p className="text-sm text-destructive">
                  {form.formState.errors.razonSocial.message}
                </p>
              )}
            </div>
            <div className="space-y-1.5">
              <Label htmlFor="nit">NIT</Label>
              <Input id="nit" {...form.register("nit")} />
            </div>
            <div className="space-y-1.5">
              <Label htmlFor="contacto">Contacto</Label>
              <Input id="contacto" {...form.register("contacto")} />
            </div>
            <div className="space-y-1.5">
              <Label htmlFor="telefono">Teléfono</Label>
              <Input id="telefono" {...form.register("telefono")} />
            </div>
            <div className="space-y-1.5">
              <Label htmlFor="correo">Correo</Label>
              <Input id="correo" type="email" {...form.register("correo")} />
            </div>
            <div className="col-span-2 space-y-1.5">
              <Label htmlFor="direccion">Dirección</Label>
              <Input id="direccion" {...form.register("direccion")} />
            </div>
            <div className="col-span-2 space-y-1.5">
              <Label htmlFor="condicionesPago">Condiciones de pago</Label>
              <Input id="condicionesPago" {...form.register("condicionesPago")} />
            </div>
          </div>

          <div className="flex items-center justify-between rounded-md border border-border p-3">
            <p className="text-sm font-medium">Activo</p>
            <Switch checked={activo} onCheckedChange={(v) => form.setValue("activo", v)} />
          </div>

          <DialogFooter>
            <Button type="button" variant="outline" onClick={() => onOpenChange(false)}>
              Cancelar
            </Button>
            <Button type="submit" disabled={mutacion.isPending}>
              {proveedor ? "Guardar cambios" : "Crear proveedor"}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  );
}

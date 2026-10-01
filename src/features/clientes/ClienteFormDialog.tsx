import { useEffect } from "react";
import { useForm } from "react-hook-form";
import { zodResolver } from "@hookform/resolvers/zod";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import { toast } from "sonner";
import { apiVentas, type Cliente } from "@/lib/api-ventas";
import {
  clienteSchema,
  valoresPorDefecto,
  type ClienteFormValues,
} from "@/features/clientes/schema";
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

function aFormulario(c: Cliente): ClienteFormValues {
  return {
    documento: c.documento ?? "",
    nombre: c.nombre,
    telefono: c.telefono ?? "",
    correo: c.correo ?? "",
    direccion: c.direccion ?? "",
  };
}

interface Props {
  abierto: boolean;
  onOpenChange: (abierto: boolean) => void;
  cliente: Cliente | null;
}

export function ClienteFormDialog({ abierto, onOpenChange, cliente }: Props) {
  const queryClient = useQueryClient();

  const form = useForm<ClienteFormValues>({
    resolver: zodResolver(clienteSchema),
    defaultValues: valoresPorDefecto,
  });

  useEffect(() => {
    if (abierto) {
      form.reset(cliente ? aFormulario(cliente) : valoresPorDefecto);
    }
  }, [abierto, cliente, form]);

  const mutacion = useMutation({
    mutationFn: (values: ClienteFormValues) => {
      const datos = {
        documento: values.documento.trim() || null,
        nombre: values.nombre.trim(),
        telefono: values.telefono.trim() || null,
        correo: values.correo.trim() || null,
        direccion: values.direccion.trim() || null,
      };
      return cliente ? apiVentas.clienteUpdate(cliente.id, datos) : apiVentas.clienteCreate(datos);
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["clientes"] });
      toast.success(cliente ? "Cliente actualizado." : "Cliente creado.");
      onOpenChange(false);
    },
    onError: (err) => toast.error(String(err)),
  });

  return (
    <Dialog open={abierto} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-lg">
        <DialogHeader>
          <DialogTitle>{cliente ? "Editar cliente" : "Nuevo cliente"}</DialogTitle>
        </DialogHeader>

        <form onSubmit={form.handleSubmit((v) => mutacion.mutate(v))} className="space-y-4">
          <div className="grid grid-cols-2 gap-4">
            <div className="col-span-2 space-y-1.5">
              <Label htmlFor="nombre">Nombre</Label>
              <Input id="nombre" {...form.register("nombre")} />
              {form.formState.errors.nombre && (
                <p className="text-sm text-destructive">{form.formState.errors.nombre.message}</p>
              )}
            </div>
            <div className="space-y-1.5">
              <Label htmlFor="documento">Documento</Label>
              <Input id="documento" {...form.register("documento")} />
            </div>
            <div className="space-y-1.5">
              <Label htmlFor="telefono">Teléfono</Label>
              <Input id="telefono" {...form.register("telefono")} />
            </div>
            <div className="space-y-1.5">
              <Label htmlFor="correo">Correo</Label>
              <Input id="correo" type="email" {...form.register("correo")} />
            </div>
            <div className="space-y-1.5">
              <Label htmlFor="direccion">Dirección</Label>
              <Input id="direccion" {...form.register("direccion")} />
            </div>
          </div>

          <DialogFooter>
            <Button type="button" variant="outline" onClick={() => onOpenChange(false)}>
              Cancelar
            </Button>
            <Button type="submit" disabled={mutacion.isPending}>
              {cliente ? "Guardar cambios" : "Crear cliente"}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  );
}

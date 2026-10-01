import { useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { toast } from "sonner";
import { apiInventario, type Proveedor } from "@/lib/api-inventario";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
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
import { ProveedorFormDialog } from "@/features/proveedores/ProveedorFormDialog";
import { HistorialComprasDialog } from "@/features/proveedores/HistorialComprasDialog";
import { ProductosQueSuministraDialog } from "@/features/proveedores/ProductosQueSuministraDialog";

export function ProveedoresPanel() {
  const queryClient = useQueryClient();
  const { data: proveedores, isLoading } = useQuery({
    queryKey: ["proveedores"],
    queryFn: apiInventario.proveedorList,
  });

  const [formAbierto, setFormAbierto] = useState(false);
  const [historialAbierto, setHistorialAbierto] = useState(false);
  const [productosAbierto, setProductosAbierto] = useState(false);
  const [seleccionado, setSeleccionado] = useState<Proveedor | null>(null);
  const [aEliminar, setAEliminar] = useState<Proveedor | null>(null);

  const eliminarMutacion = useMutation({
    mutationFn: (id: number) => apiInventario.proveedorDelete(id),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["proveedores"] });
      toast.success("Proveedor eliminado.");
      setAEliminar(null);
    },
    onError: (err) => {
      toast.error(String(err));
      setAEliminar(null);
    },
  });

  return (
    <div className="space-y-4">
      <div className="flex justify-end">
        <Button
          onClick={() => {
            setSeleccionado(null);
            setFormAbierto(true);
          }}
        >
          Nuevo proveedor
        </Button>
      </div>

      <div className="rounded-md border border-border">
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead>Razón social</TableHead>
              <TableHead>NIT</TableHead>
              <TableHead>Contacto</TableHead>
              <TableHead>Teléfono</TableHead>
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
            {!isLoading && proveedores?.length === 0 && (
              <TableRow>
                <TableCell colSpan={6} className="text-center text-muted-foreground">
                  No hay proveedores registrados.
                </TableCell>
              </TableRow>
            )}
            {proveedores?.map((p) => (
              <TableRow key={p.id}>
                <TableCell>{p.razonSocial}</TableCell>
                <TableCell className="text-muted-foreground">{p.nit ?? "—"}</TableCell>
                <TableCell className="text-muted-foreground">{p.contacto ?? "—"}</TableCell>
                <TableCell className="text-muted-foreground">{p.telefono ?? "—"}</TableCell>
                <TableCell>
                  <Badge variant={p.activo ? "success" : "secondary"}>
                    {p.activo ? "activo" : "inactivo"}
                  </Badge>
                </TableCell>
                <TableCell className="space-x-2 text-right">
                  <Button
                    variant="outline"
                    size="sm"
                    onClick={() => {
                      setSeleccionado(p);
                      setHistorialAbierto(true);
                    }}
                  >
                    Compras
                  </Button>
                  <Button
                    variant="outline"
                    size="sm"
                    onClick={() => {
                      setSeleccionado(p);
                      setProductosAbierto(true);
                    }}
                  >
                    Productos
                  </Button>
                  <Button
                    variant="outline"
                    size="sm"
                    onClick={() => {
                      setSeleccionado(p);
                      setFormAbierto(true);
                    }}
                  >
                    Editar
                  </Button>
                  <Button variant="destructive" size="sm" onClick={() => setAEliminar(p)}>
                    Eliminar
                  </Button>
                </TableCell>
              </TableRow>
            ))}
          </TableBody>
        </Table>
      </div>

      <ProveedorFormDialog
        abierto={formAbierto}
        onOpenChange={setFormAbierto}
        proveedor={seleccionado}
      />
      <HistorialComprasDialog
        abierto={historialAbierto}
        onOpenChange={setHistorialAbierto}
        proveedor={seleccionado}
      />
      <ProductosQueSuministraDialog
        abierto={productosAbierto}
        onOpenChange={setProductosAbierto}
        proveedor={seleccionado}
      />

      <AlertDialog open={!!aEliminar} onOpenChange={(v) => !v && setAEliminar(null)}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>¿Eliminar "{aEliminar?.razonSocial}"?</AlertDialogTitle>
            <AlertDialogDescription>
              Los productos y compras asociados quedan sin proveedor (no se eliminan).
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>Cancelar</AlertDialogCancel>
            <AlertDialogAction onClick={() => aEliminar && eliminarMutacion.mutate(aEliminar.id)}>
              Eliminar
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </div>
  );
}

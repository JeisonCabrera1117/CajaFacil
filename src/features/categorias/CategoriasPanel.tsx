import { useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { toast } from "sonner";
import { apiInventario, type Categoria } from "@/lib/api-inventario";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";

export function CategoriasPanel() {
  const queryClient = useQueryClient();
  const { data: categorias, isLoading } = useQuery({
    queryKey: ["categorias"],
    queryFn: apiInventario.categoriaList,
  });

  const [nombre, setNombre] = useState("");
  const [padreId, setPadreId] = useState("none");
  const [editando, setEditando] = useState<Categoria | null>(null);

  const crearMutacion = useMutation({
    mutationFn: () =>
      apiInventario.categoriaCreate({
        nombre: nombre.trim(),
        categoriaPadreId: padreId !== "none" ? Number(padreId) : null,
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["categorias"] });
      setNombre("");
      setPadreId("none");
      toast.success("Categoría creada.");
    },
    onError: (err) => toast.error(String(err)),
  });

  const actualizarMutacion = useMutation({
    mutationFn: () =>
      apiInventario.categoriaUpdate(editando!.id, {
        nombre: nombre.trim(),
        categoriaPadreId: padreId !== "none" ? Number(padreId) : null,
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["categorias"] });
      setEditando(null);
      setNombre("");
      setPadreId("none");
      toast.success("Categoría actualizada.");
    },
    onError: (err) => toast.error(String(err)),
  });

  const eliminarMutacion = useMutation({
    mutationFn: (id: number) => apiInventario.categoriaDelete(id),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["categorias"] });
      toast.success("Categoría eliminada.");
    },
    onError: (err) => toast.error(String(err)),
  });

  function editar(categoria: Categoria) {
    setEditando(categoria);
    setNombre(categoria.nombre);
    setPadreId(categoria.categoriaPadreId ? String(categoria.categoriaPadreId) : "none");
  }

  function cancelarEdicion() {
    setEditando(null);
    setNombre("");
    setPadreId("none");
  }

  function nombrePadre(id: number | null): string {
    if (!id) return "—";
    return categorias?.find((c) => c.id === id)?.nombre ?? "—";
  }

  return (
    <div className="grid max-w-3xl gap-6 md:grid-cols-2">
      <Card>
        <CardHeader>
          <CardTitle>{editando ? "Editar categoría" : "Nueva categoría"}</CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="space-y-1.5">
            <Label htmlFor="nombreCategoria">Nombre</Label>
            <Input
              id="nombreCategoria"
              value={nombre}
              onChange={(e) => setNombre(e.currentTarget.value)}
            />
          </div>
          <div className="space-y-1.5">
            <Label>Categoría padre (opcional)</Label>
            <Select value={padreId} onValueChange={(v) => setPadreId(v ?? "none")}>
              <SelectTrigger>
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="none">Ninguna (categoría raíz)</SelectItem>
                {categorias
                  ?.filter((c) => c.id !== editando?.id)
                  .map((c) => (
                    <SelectItem key={c.id} value={String(c.id)}>
                      {c.nombre}
                    </SelectItem>
                  ))}
              </SelectContent>
            </Select>
          </div>
          <div className="flex gap-2">
            {editando ? (
              <>
                <Button onClick={() => actualizarMutacion.mutate()} disabled={!nombre.trim()}>
                  Guardar cambios
                </Button>
                <Button variant="outline" onClick={cancelarEdicion}>
                  Cancelar
                </Button>
              </>
            ) : (
              <Button onClick={() => crearMutacion.mutate()} disabled={!nombre.trim()}>
                Crear categoría
              </Button>
            )}
          </div>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>Categorías</CardTitle>
        </CardHeader>
        <CardContent>
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>Nombre</TableHead>
                <TableHead>Padre</TableHead>
                <TableHead className="text-right">Acciones</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {isLoading && (
                <TableRow>
                  <TableCell colSpan={3} className="text-center text-muted-foreground">
                    Cargando…
                  </TableCell>
                </TableRow>
              )}
              {categorias?.map((c) => (
                <TableRow key={c.id}>
                  <TableCell>{c.nombre}</TableCell>
                  <TableCell className="text-muted-foreground">
                    {nombrePadre(c.categoriaPadreId)}
                  </TableCell>
                  <TableCell className="space-x-2 text-right">
                    <Button variant="outline" size="sm" onClick={() => editar(c)}>
                      Editar
                    </Button>
                    <Button
                      variant="destructive"
                      size="sm"
                      onClick={() => eliminarMutacion.mutate(c.id)}
                    >
                      Eliminar
                    </Button>
                  </TableCell>
                </TableRow>
              ))}
            </TableBody>
          </Table>
        </CardContent>
      </Card>
    </div>
  );
}

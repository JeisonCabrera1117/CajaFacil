import { useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { toast } from "sonner";
import { apiInventario, type Producto } from "@/lib/api-inventario";
import { formatearMoneda } from "@/lib/format";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Badge } from "@/components/ui/badge";
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
import { ProductoFormDialog } from "@/features/productos/ProductoFormDialog";
import { KardexDialog } from "@/features/productos/KardexDialog";

const POR_PAGINA = 20;

export function ProductosPanel() {
  const queryClient = useQueryClient();
  const [busqueda, setBusqueda] = useState("");
  const [soloStockBajo, setSoloStockBajo] = useState(false);
  const [pagina, setPagina] = useState(1);

  const [dialogoAbierto, setDialogoAbierto] = useState(false);
  const [kardexAbierto, setKardexAbierto] = useState(false);
  const [productoSeleccionado, setProductoSeleccionado] = useState<Producto | null>(null);
  const [productoAEliminar, setProductoAEliminar] = useState<Producto | null>(null);

  const { data, isLoading } = useQuery({
    queryKey: ["productos", { busqueda, soloStockBajo, pagina }],
    queryFn: () =>
      apiInventario.productoList({
        busqueda: busqueda || undefined,
        soloStockBajo: soloStockBajo || undefined,
        pagina,
        porPagina: POR_PAGINA,
      }),
  });

  const eliminarMutacion = useMutation({
    mutationFn: (id: number) => apiInventario.productoDelete(id),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["productos"] });
      toast.success("Producto eliminado.");
      setProductoAEliminar(null);
    },
    onError: (err) => {
      toast.error(String(err));
      setProductoAEliminar(null);
    },
  });

  const totalPaginas = data ? Math.max(1, Math.ceil(data.total / POR_PAGINA)) : 1;

  return (
    <div className="space-y-4">
      <div className="flex flex-wrap items-center gap-3">
        <Input
          placeholder="Buscar por nombre, SKU o código de barras…"
          value={busqueda}
          onChange={(e) => {
            setBusqueda(e.currentTarget.value);
            setPagina(1);
          }}
          className="max-w-xs"
        />
        <Button
          variant={soloStockBajo ? "default" : "outline"}
          size="sm"
          onClick={() => {
            setSoloStockBajo((v) => !v);
            setPagina(1);
          }}
        >
          Stock bajo
        </Button>
        <div className="flex-1" />
        <Button
          onClick={() => {
            setProductoSeleccionado(null);
            setDialogoAbierto(true);
          }}
        >
          Nuevo producto
        </Button>
      </div>

      <div className="rounded-md border border-border">
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead>SKU</TableHead>
              <TableHead>Nombre</TableHead>
              <TableHead className="text-right">Stock</TableHead>
              <TableHead className="text-right">Costo</TableHead>
              <TableHead className="text-right">Venta</TableHead>
              <TableHead>Estado</TableHead>
              <TableHead className="text-right">Acciones</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {isLoading && (
              <TableRow>
                <TableCell colSpan={7} className="text-center text-muted-foreground">
                  Cargando…
                </TableCell>
              </TableRow>
            )}
            {!isLoading && data?.items.length === 0 && (
              <TableRow>
                <TableCell colSpan={7} className="text-center text-muted-foreground">
                  No hay productos que coincidan.
                </TableCell>
              </TableRow>
            )}
            {data?.items.map((p) => (
              <TableRow key={p.id}>
                <TableCell className="font-mono text-sm">{p.sku}</TableCell>
                <TableCell>{p.nombre}</TableCell>
                <TableCell className="text-right">
                  <button
                    className="underline decoration-dotted underline-offset-2"
                    onClick={() => {
                      setProductoSeleccionado(p);
                      setKardexAbierto(true);
                    }}
                  >
                    {p.stockActual}
                  </button>
                  {p.stockActual <= p.stockMinimo && (
                    <Badge
                      variant={p.stockActual === 0 ? "destructive" : "warning"}
                      className="ml-2"
                    >
                      bajo
                    </Badge>
                  )}
                </TableCell>
                <TableCell className="text-right">{formatearMoneda(p.precioCosto)}</TableCell>
                <TableCell className="text-right">{formatearMoneda(p.precioVenta)}</TableCell>
                <TableCell>
                  <Badge variant={p.estado === "activo" ? "success" : "secondary"}>
                    {p.estado}
                  </Badge>
                </TableCell>
                <TableCell className="text-right space-x-2">
                  <Button
                    variant="outline"
                    size="sm"
                    onClick={() => {
                      setProductoSeleccionado(p);
                      setDialogoAbierto(true);
                    }}
                  >
                    Editar
                  </Button>
                  <Button variant="destructive" size="sm" onClick={() => setProductoAEliminar(p)}>
                    Eliminar
                  </Button>
                </TableCell>
              </TableRow>
            ))}
          </TableBody>
        </Table>
      </div>

      {data && data.total > POR_PAGINA && (
        <div className="flex items-center justify-between">
          <p className="text-sm text-muted-foreground">
            Página {pagina} de {totalPaginas} — {data.total} productos
          </p>
          <div className="space-x-2">
            <Button
              variant="outline"
              size="sm"
              disabled={pagina <= 1}
              onClick={() => setPagina((p) => Math.max(1, p - 1))}
            >
              Anterior
            </Button>
            <Button
              variant="outline"
              size="sm"
              disabled={pagina >= totalPaginas}
              onClick={() => setPagina((p) => p + 1)}
            >
              Siguiente
            </Button>
          </div>
        </div>
      )}

      <ProductoFormDialog
        abierto={dialogoAbierto}
        onOpenChange={setDialogoAbierto}
        producto={productoSeleccionado}
      />
      <KardexDialog
        abierto={kardexAbierto}
        onOpenChange={setKardexAbierto}
        producto={productoSeleccionado}
      />

      <AlertDialog
        open={!!productoAEliminar}
        onOpenChange={(v) => !v && setProductoAEliminar(null)}
      >
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>¿Eliminar "{productoAEliminar?.nombre}"?</AlertDialogTitle>
            <AlertDialogDescription>
              Si el producto ya tiene movimientos, compras o ventas asociadas no se podrá eliminar;
              en ese caso, márquelo como inactivo desde "Editar" en su lugar.
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>Cancelar</AlertDialogCancel>
            <AlertDialogAction
              onClick={() => productoAEliminar && eliminarMutacion.mutate(productoAEliminar.id)}
            >
              Eliminar
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </div>
  );
}

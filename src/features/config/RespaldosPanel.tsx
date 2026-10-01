import { useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { toast } from "sonner";
import { apiRespaldos, type Respaldo } from "@/lib/api-respaldos";
import { apiComprobantes } from "@/lib/api-comprobantes";
import { formatearTamano, formatearFechaHora } from "@/lib/format";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from "@/components/ui/card";
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

export function RespaldosPanel() {
  const queryClient = useQueryClient();
  const [respaldoARestaurar, setRespaldoARestaurar] = useState<Respaldo | null>(null);
  const [respaldoAEliminar, setRespaldoAEliminar] = useState<Respaldo | null>(null);

  const { data: respaldos, isLoading } = useQuery({
    queryKey: ["respaldos"],
    queryFn: apiRespaldos.list,
  });

  const crear = useMutation({
    mutationFn: apiRespaldos.crear,
    onSuccess: () => {
      toast.success("Respaldo creado.");
      queryClient.invalidateQueries({ queryKey: ["respaldos"] });
    },
    onError: (err) => toast.error(String(err)),
  });

  const eliminar = useMutation({
    mutationFn: apiRespaldos.eliminar,
    onSuccess: () => {
      toast.success("Respaldo eliminado.");
      queryClient.invalidateQueries({ queryKey: ["respaldos"] });
      setRespaldoAEliminar(null);
    },
    onError: (err) => toast.error(String(err)),
  });

  const restaurar = useMutation({
    mutationFn: apiRespaldos.restaurar,
    onError: (err) => toast.error(String(err)),
  });

  async function abrirCarpeta() {
    if (!respaldos || respaldos.length === 0) {
      toast.info("Todavía no hay ningún respaldo creado.");
      return;
    }
    try {
      const carpeta = respaldos[0].rutaArchivo.replace(/[\\/][^\\/]+$/, "");
      await apiComprobantes.abrir(carpeta);
    } catch (err) {
      toast.error(String(err));
    }
  }

  return (
    <Card>
      <CardHeader>
        <CardTitle>Respaldos</CardTitle>
        <CardDescription>
          Copias de la base de datos que se pueden restaurar más adelante. Se conservan los últimos
          20 respaldos; los más viejos se eliminan automáticamente.
        </CardDescription>
      </CardHeader>
      <CardContent className="space-y-4">
        <div className="flex gap-2">
          <Button onClick={() => crear.mutate()} disabled={crear.isPending}>
            {crear.isPending ? "Creando…" : "Crear respaldo ahora"}
          </Button>
          <Button variant="outline" onClick={abrirCarpeta}>
            Abrir carpeta
          </Button>
        </div>

        <div className="rounded-md border border-border">
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>Fecha</TableHead>
                <TableHead>Tipo</TableHead>
                <TableHead className="text-right">Tamaño</TableHead>
                <TableHead></TableHead>
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
              {!isLoading && respaldos?.length === 0 && (
                <TableRow>
                  <TableCell colSpan={4} className="text-center text-muted-foreground">
                    Todavía no se ha creado ningún respaldo.
                  </TableCell>
                </TableRow>
              )}
              {respaldos?.map((r) => (
                <TableRow key={r.id}>
                  <TableCell className="whitespace-nowrap text-sm">
                    {formatearFechaHora(r.fecha)}
                  </TableCell>
                  <TableCell>
                    <Badge variant="outline">{r.tipo}</Badge>
                  </TableCell>
                  <TableCell className="text-right text-sm">
                    {formatearTamano(r.tamanoBytes)}
                  </TableCell>
                  <TableCell className="flex justify-end gap-2">
                    <Button variant="outline" size="sm" onClick={() => setRespaldoARestaurar(r)}>
                      Restaurar
                    </Button>
                    <Button
                      variant="ghost"
                      size="sm"
                      className="text-destructive"
                      onClick={() => setRespaldoAEliminar(r)}
                    >
                      Eliminar
                    </Button>
                  </TableCell>
                </TableRow>
              ))}
            </TableBody>
          </Table>
        </div>
      </CardContent>

      <AlertDialog
        open={respaldoARestaurar !== null}
        onOpenChange={(v) => !v && setRespaldoARestaurar(null)}
      >
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>¿Restaurar este respaldo?</AlertDialogTitle>
            <AlertDialogDescription>
              Todos los cambios hechos después del{" "}
              {respaldoARestaurar && formatearFechaHora(respaldoARestaurar.fecha)} se perderán. La
              aplicación se cerrará sola para completar la restauración: vuelva a abrirla después.
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>Cancelar</AlertDialogCancel>
            <AlertDialogAction
              onClick={() => respaldoARestaurar && restaurar.mutate(respaldoARestaurar.id)}
            >
              Restaurar y cerrar
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>

      <AlertDialog
        open={respaldoAEliminar !== null}
        onOpenChange={(v) => !v && setRespaldoAEliminar(null)}
      >
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>¿Eliminar este respaldo?</AlertDialogTitle>
            <AlertDialogDescription>
              Esta acción no se puede deshacer. El archivo del{" "}
              {respaldoAEliminar && formatearFechaHora(respaldoAEliminar.fecha)} se borrará del
              disco.
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>Cancelar</AlertDialogCancel>
            <AlertDialogAction
              onClick={() => respaldoAEliminar && eliminar.mutate(respaldoAEliminar.id)}
            >
              Eliminar
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </Card>
  );
}

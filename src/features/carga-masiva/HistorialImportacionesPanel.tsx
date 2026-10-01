import { useQuery } from "@tanstack/react-query";
import { toast } from "sonner";
import { apiImportacion } from "@/lib/api-importacion";
import { apiComprobantes } from "@/lib/api-comprobantes";
import { formatearFechaHora } from "@/lib/format";
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

export function HistorialImportacionesPanel() {
  const { data: historial, isLoading } = useQuery({
    queryKey: ["import-historial"],
    queryFn: apiImportacion.historial,
  });

  async function abrirRechazados(ruta: string) {
    try {
      await apiComprobantes.abrir(ruta);
    } catch (err) {
      toast.error(String(err));
    }
  }

  return (
    <div className="rounded-md border border-border">
      <Table>
        <TableHeader>
          <TableRow>
            <TableHead>Fecha</TableHead>
            <TableHead>Entidad</TableHead>
            <TableHead>Archivo</TableHead>
            <TableHead>Modo</TableHead>
            <TableHead className="text-right">Filas</TableHead>
            <TableHead className="text-right">Creados</TableHead>
            <TableHead className="text-right">Actualizados</TableHead>
            <TableHead className="text-right">Con error</TableHead>
            <TableHead></TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          {isLoading && (
            <TableRow>
              <TableCell colSpan={9} className="text-center text-muted-foreground">
                Cargando…
              </TableCell>
            </TableRow>
          )}
          {!isLoading && historial?.length === 0 && (
            <TableRow>
              <TableCell colSpan={9} className="text-center text-muted-foreground">
                Todavía no se hizo ninguna importación.
              </TableCell>
            </TableRow>
          )}
          {historial?.map((h) => (
            <TableRow key={h.id}>
              <TableCell className="whitespace-nowrap text-sm">
                {formatearFechaHora(h.fecha)}
              </TableCell>
              <TableCell>{h.entidad}</TableCell>
              <TableCell className="max-w-40 truncate text-sm text-muted-foreground">
                {h.archivoNombre}
              </TableCell>
              <TableCell>
                <Badge variant="outline">{h.modo}</Badge>
              </TableCell>
              <TableCell className="text-right">{h.totalFilas}</TableCell>
              <TableCell className="text-right">{h.creados}</TableCell>
              <TableCell className="text-right">{h.actualizados}</TableCell>
              <TableCell className="text-right">
                {h.conError > 0 ? <Badge variant="warning">{h.conError}</Badge> : "0"}
              </TableCell>
              <TableCell>
                {h.archivoRechazadosPath && (
                  <Button
                    variant="outline"
                    size="sm"
                    onClick={() => abrirRechazados(h.archivoRechazadosPath!)}
                  >
                    Ver rechazados
                  </Button>
                )}
              </TableCell>
            </TableRow>
          ))}
        </TableBody>
      </Table>
    </div>
  );
}

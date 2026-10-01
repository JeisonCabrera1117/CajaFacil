import { useState } from "react";
import { useQuery } from "@tanstack/react-query";
import { apiAuditoria, ENTIDADES_AUDITORIA } from "@/lib/api-auditoria";
import { formatearFechaHora } from "@/lib/format";
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
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

const POR_PAGINA = 50;

const ETIQUETA_ACCION: Record<
  string,
  "default" | "secondary" | "destructive" | "outline" | "success" | "warning"
> = {
  crear: "success",
  modificar: "secondary",
  anular: "warning",
  eliminar: "destructive",
};

export function AuditoriaPanel() {
  const [entidad, setEntidad] = useState("todas");
  const [desde, setDesde] = useState("");
  const [hasta, setHasta] = useState("");
  const [pagina, setPagina] = useState(1);

  const { data: entradas, isLoading } = useQuery({
    queryKey: ["auditoria", { entidad, desde, hasta, pagina }],
    queryFn: () =>
      apiAuditoria.list({
        entidad: entidad !== "todas" ? entidad : null,
        desde: desde || null,
        hasta: hasta || null,
        pagina,
        porPagina: POR_PAGINA,
      }),
  });

  function actualizarFiltro(fn: () => void) {
    fn();
    setPagina(1);
  }

  return (
    <Card>
      <CardHeader>
        <CardTitle>Historial de cambios</CardTitle>
        <CardDescription>
          Registro de auditoría: quién hizo qué y cuándo, para las entidades principales.
        </CardDescription>
      </CardHeader>
      <CardContent className="space-y-4">
        <div className="flex flex-wrap items-end gap-3">
          <div className="space-y-1.5">
            <Label>Entidad</Label>
            <Select
              value={entidad}
              onValueChange={(v) => v && actualizarFiltro(() => setEntidad(v))}
            >
              <SelectTrigger className="w-44">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="todas">Todas</SelectItem>
                {ENTIDADES_AUDITORIA.map((e) => (
                  <SelectItem key={e} value={e}>
                    {e}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>
          <div className="space-y-1.5">
            <Label>Desde</Label>
            <Input
              type="date"
              value={desde}
              onChange={(e) => actualizarFiltro(() => setDesde(e.currentTarget.value))}
            />
          </div>
          <div className="space-y-1.5">
            <Label>Hasta</Label>
            <Input
              type="date"
              value={hasta}
              onChange={(e) => actualizarFiltro(() => setHasta(e.currentTarget.value))}
            />
          </div>
        </div>

        <div className="max-h-[28rem] overflow-y-auto rounded-md border border-border">
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>Fecha</TableHead>
                <TableHead>Entidad</TableHead>
                <TableHead>Acción</TableHead>
                <TableHead>Detalle</TableHead>
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
              {!isLoading && entradas?.length === 0 && (
                <TableRow>
                  <TableCell colSpan={4} className="text-center text-muted-foreground">
                    Sin registros para este filtro.
                  </TableCell>
                </TableRow>
              )}
              {entradas?.map((e) => (
                <TableRow key={e.id}>
                  <TableCell className="whitespace-nowrap text-sm">
                    {formatearFechaHora(e.fecha)}
                  </TableCell>
                  <TableCell>
                    {e.entidad}
                    {e.entidadId !== null && (
                      <span className="text-muted-foreground"> #{e.entidadId}</span>
                    )}
                  </TableCell>
                  <TableCell>
                    <Badge variant={ETIQUETA_ACCION[e.accion] ?? "outline"}>{e.accion}</Badge>
                  </TableCell>
                  <TableCell
                    className="max-w-96 truncate font-mono text-xs text-muted-foreground"
                    title={e.detalleJson ?? undefined}
                  >
                    {e.detalleJson ?? "—"}
                  </TableCell>
                </TableRow>
              ))}
            </TableBody>
          </Table>
        </div>

        <div className="flex items-center justify-end gap-2">
          <Button
            variant="outline"
            size="sm"
            disabled={pagina === 1}
            onClick={() => setPagina((p) => p - 1)}
          >
            Anterior
          </Button>
          <span className="text-sm text-muted-foreground">Página {pagina}</span>
          <Button
            variant="outline"
            size="sm"
            disabled={(entradas?.length ?? 0) < POR_PAGINA}
            onClick={() => setPagina((p) => p + 1)}
          >
            Siguiente
          </Button>
        </div>
      </CardContent>
    </Card>
  );
}

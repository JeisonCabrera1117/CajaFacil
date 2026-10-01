import { useState } from "react";
import { useQuery } from "@tanstack/react-query";
import { apiVentas } from "@/lib/api-ventas";
import { formatearMoneda, formatearFechaHora } from "@/lib/format";
import { Badge } from "@/components/ui/badge";
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
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import { VentaDetalleDialog } from "@/features/ventas/VentaDetalleDialog";

export function HistorialVentasPanel() {
  const [desde, setDesde] = useState("");
  const [hasta, setHasta] = useState("");
  const [estado, setEstado] = useState("todos");
  const [metodoPago, setMetodoPago] = useState("todos");
  const [ventaSeleccionada, setVentaSeleccionada] = useState<number | null>(null);

  const { data: ventas, isLoading } = useQuery({
    queryKey: ["ventas", { desde, hasta, estado, metodoPago }],
    queryFn: () =>
      apiVentas.ventaList({
        desde: desde || undefined,
        hasta: hasta || undefined,
        estado: estado !== "todos" ? estado : undefined,
        metodoPago: metodoPago !== "todos" ? metodoPago : undefined,
      }),
  });

  return (
    <div className="space-y-4">
      <div className="flex flex-wrap items-end gap-3">
        <div className="space-y-1.5">
          <Label htmlFor="desde">Desde</Label>
          <Input
            id="desde"
            type="date"
            value={desde}
            onChange={(e) => setDesde(e.currentTarget.value)}
          />
        </div>
        <div className="space-y-1.5">
          <Label htmlFor="hasta">Hasta</Label>
          <Input
            id="hasta"
            type="date"
            value={hasta}
            onChange={(e) => setHasta(e.currentTarget.value)}
          />
        </div>
        <div className="space-y-1.5">
          <Label>Estado</Label>
          <Select value={estado} onValueChange={(v) => setEstado(v ?? "todos")}>
            <SelectTrigger className="w-40">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="todos">Todos</SelectItem>
              <SelectItem value="completada">Completada</SelectItem>
              <SelectItem value="anulada">Anulada</SelectItem>
            </SelectContent>
          </Select>
        </div>
        <div className="space-y-1.5">
          <Label>Método de pago</Label>
          <Select value={metodoPago} onValueChange={(v) => setMetodoPago(v ?? "todos")}>
            <SelectTrigger className="w-44">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="todos">Todos</SelectItem>
              <SelectItem value="efectivo">Efectivo</SelectItem>
              <SelectItem value="tarjeta">Tarjeta</SelectItem>
              <SelectItem value="transferencia">Transferencia</SelectItem>
              <SelectItem value="mixto">Mixto</SelectItem>
            </SelectContent>
          </Select>
        </div>
      </div>

      <div className="rounded-md border border-border">
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead>Comprobante</TableHead>
              <TableHead>Fecha</TableHead>
              <TableHead>Cliente</TableHead>
              <TableHead>Pago</TableHead>
              <TableHead className="text-right">Total</TableHead>
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
            {!isLoading && ventas?.length === 0 && (
              <TableRow>
                <TableCell colSpan={7} className="text-center text-muted-foreground">
                  No hay ventas que coincidan con el filtro.
                </TableCell>
              </TableRow>
            )}
            {ventas?.map((v) => (
              <TableRow key={v.id}>
                <TableCell className="font-mono text-sm">{v.numeroComprobante}</TableCell>
                <TableCell className="whitespace-nowrap text-sm">
                  {formatearFechaHora(v.fecha)}
                </TableCell>
                <TableCell>{v.clienteNombre ?? "General"}</TableCell>
                <TableCell className="capitalize">{v.metodoPago}</TableCell>
                <TableCell className="text-right">{formatearMoneda(v.total)}</TableCell>
                <TableCell>
                  <Badge variant={v.estado === "completada" ? "success" : "destructive"}>
                    {v.estado}
                  </Badge>
                </TableCell>
                <TableCell className="text-right">
                  <Button variant="outline" size="sm" onClick={() => setVentaSeleccionada(v.id)}>
                    Ver
                  </Button>
                </TableCell>
              </TableRow>
            ))}
          </TableBody>
        </Table>
      </div>

      <VentaDetalleDialog
        ventaId={ventaSeleccionada}
        onOpenChange={(v) => !v && setVentaSeleccionada(null)}
      />
    </div>
  );
}

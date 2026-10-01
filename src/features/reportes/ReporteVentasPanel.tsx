import { useState } from "react";
import { useQuery } from "@tanstack/react-query";
import { apiReportes } from "@/lib/api-reportes";
import { apiInventario } from "@/lib/api-inventario";
import { apiVentas } from "@/lib/api-ventas";
import { formatearMoneda, formatearFechaHora, primerDiaMesISO, hoyISO } from "@/lib/format";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Badge } from "@/components/ui/badge";
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
import { ExportarReporteBotones } from "@/features/reportes/ExportarReporteBotones";

const COLUMNAS = [
  "Fecha",
  "Comprobante",
  "Cliente",
  "Producto",
  "Categoría",
  "Cantidad",
  "Precio unitario",
  "Subtotal",
  "Estado",
];

export function ReporteVentasPanel() {
  const [desde, setDesde] = useState(primerDiaMesISO());
  const [hasta, setHasta] = useState(hoyISO());
  const [productoId, setProductoId] = useState("todos");
  const [categoriaId, setCategoriaId] = useState("todas");
  const [clienteId, setClienteId] = useState("todos");

  const { data: categorias } = useQuery({
    queryKey: ["categorias"],
    queryFn: apiInventario.categoriaList,
  });
  const { data: clientes } = useQuery({ queryKey: ["clientes"], queryFn: apiVentas.clienteList });
  const { data: catalogo } = useQuery({
    queryKey: ["productos", "catalogo-completo"],
    queryFn: () => apiInventario.productoList({ porPagina: 500, pagina: 1 }),
  });

  const { data: filas, isLoading } = useQuery({
    queryKey: ["reporte-ventas", { desde, hasta, productoId, categoriaId, clienteId }],
    queryFn: () =>
      apiReportes.ventas({
        desde,
        hasta,
        productoId: productoId !== "todos" ? Number(productoId) : null,
        categoriaId: categoriaId !== "todas" ? Number(categoriaId) : null,
        clienteId: clienteId !== "todos" ? Number(clienteId) : null,
      }),
  });

  const filasExportar = (filas ?? []).map((f) => [
    formatearFechaHora(f.fecha),
    f.numeroComprobante,
    f.cliente,
    f.producto,
    f.categoria ?? "",
    String(f.cantidad),
    formatearMoneda(f.precioUnitario),
    formatearMoneda(f.subtotal),
    f.estado,
  ]);

  return (
    <div className="space-y-4">
      <div className="flex flex-wrap items-end gap-3">
        <div className="space-y-1.5">
          <Label>Desde</Label>
          <Input type="date" value={desde} onChange={(e) => setDesde(e.currentTarget.value)} />
        </div>
        <div className="space-y-1.5">
          <Label>Hasta</Label>
          <Input type="date" value={hasta} onChange={(e) => setHasta(e.currentTarget.value)} />
        </div>
        <div className="space-y-1.5">
          <Label>Producto</Label>
          <Select value={productoId} onValueChange={(v) => v && setProductoId(v)}>
            <SelectTrigger className="w-48">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="todos">Todos</SelectItem>
              {catalogo?.items.map((p) => (
                <SelectItem key={p.id} value={String(p.id)}>
                  {p.nombre}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </div>
        <div className="space-y-1.5">
          <Label>Categoría</Label>
          <Select value={categoriaId} onValueChange={(v) => v && setCategoriaId(v)}>
            <SelectTrigger className="w-40">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="todas">Todas</SelectItem>
              {categorias?.map((c) => (
                <SelectItem key={c.id} value={String(c.id)}>
                  {c.nombre}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </div>
        <div className="space-y-1.5">
          <Label>Cliente</Label>
          <Select value={clienteId} onValueChange={(v) => v && setClienteId(v)}>
            <SelectTrigger className="w-40">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="todos">Todos</SelectItem>
              {clientes?.map((c) => (
                <SelectItem key={c.id} value={String(c.id)}>
                  {c.nombre}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </div>
        <div className="flex-1" />
        <ExportarReporteBotones
          titulo="Reporte de ventas"
          columnas={COLUMNAS}
          filas={filasExportar}
        />
      </div>

      <div className="max-h-[28rem] overflow-y-auto rounded-md border border-border">
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead>Fecha</TableHead>
              <TableHead>Comprobante</TableHead>
              <TableHead>Cliente</TableHead>
              <TableHead>Producto</TableHead>
              <TableHead>Categoría</TableHead>
              <TableHead className="text-right">Cant.</TableHead>
              <TableHead className="text-right">Precio</TableHead>
              <TableHead className="text-right">Subtotal</TableHead>
              <TableHead>Estado</TableHead>
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
            {!isLoading && filas?.length === 0 && (
              <TableRow>
                <TableCell colSpan={9} className="text-center text-muted-foreground">
                  Sin resultados para este filtro.
                </TableCell>
              </TableRow>
            )}
            {filas?.map((f, i) => (
              <TableRow key={i}>
                <TableCell className="whitespace-nowrap text-sm">
                  {formatearFechaHora(f.fecha)}
                </TableCell>
                <TableCell className="font-mono text-sm">{f.numeroComprobante}</TableCell>
                <TableCell>{f.cliente}</TableCell>
                <TableCell>{f.producto}</TableCell>
                <TableCell className="text-muted-foreground">{f.categoria ?? "—"}</TableCell>
                <TableCell className="text-right">{f.cantidad}</TableCell>
                <TableCell className="text-right">{formatearMoneda(f.precioUnitario)}</TableCell>
                <TableCell className="text-right">{formatearMoneda(f.subtotal)}</TableCell>
                <TableCell>
                  <Badge variant={f.estado === "completada" ? "success" : "destructive"}>
                    {f.estado}
                  </Badge>
                </TableCell>
              </TableRow>
            ))}
          </TableBody>
        </Table>
      </div>
    </div>
  );
}

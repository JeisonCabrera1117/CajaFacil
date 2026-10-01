import { useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { toast } from "sonner";
import { Trash2 } from "lucide-react";
import { apiVentas, type MetodoPago } from "@/lib/api-ventas";
import { formatearMoneda } from "@/lib/format";
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
import { ComprobanteDialog } from "@/features/comprobantes/ComprobanteDialog";

interface ItemCarrito {
  productoId: number;
  sku: string;
  nombre: string;
  cantidad: number;
  precioUnitario: number;
  descuentoValor: number;
  impuestoPct: number;
  stockActual: number;
}

const METODOS: { value: MetodoPago; label: string }[] = [
  { value: "efectivo", label: "Efectivo" },
  { value: "tarjeta", label: "Tarjeta" },
  { value: "transferencia", label: "Transferencia" },
  { value: "mixto", label: "Mixto" },
];

export function Pos() {
  const queryClient = useQueryClient();
  const [termino, setTermino] = useState("");
  const [carrito, setCarrito] = useState<ItemCarrito[]>([]);
  const [clienteId, setClienteId] = useState("general");
  const [metodoPago, setMetodoPago] = useState<MetodoPago>("efectivo");
  const [descuentoGlobal, setDescuentoGlobal] = useState("0");
  const [valorRecibido, setValorRecibido] = useState("");
  const [ventaParaComprobante, setVentaParaComprobante] = useState<number | null>(null);

  const { data: clientes } = useQuery({ queryKey: ["clientes"], queryFn: apiVentas.clienteList });
  const { data: resultados } = useQuery({
    queryKey: ["buscar-producto", termino],
    queryFn: () => apiVentas.ventaBuscarProducto(termino),
    enabled: termino.trim().length >= 2,
  });

  function agregarAlCarrito(p: {
    id: number;
    sku: string;
    nombre: string;
    precioVenta: number;
    impuestoPct: number;
    stockActual: number;
  }) {
    setCarrito((prev) => {
      const existente = prev.find((i) => i.productoId === p.id);
      if (existente) {
        return prev.map((i) => (i.productoId === p.id ? { ...i, cantidad: i.cantidad + 1 } : i));
      }
      return [
        ...prev,
        {
          productoId: p.id,
          sku: p.sku,
          nombre: p.nombre,
          cantidad: 1,
          precioUnitario: p.precioVenta,
          descuentoValor: 0,
          impuestoPct: p.impuestoPct,
          stockActual: p.stockActual,
        },
      ];
    });
    setTermino("");
  }

  function actualizarItem(productoId: number, cambios: Partial<ItemCarrito>) {
    setCarrito((prev) => prev.map((i) => (i.productoId === productoId ? { ...i, ...cambios } : i)));
  }

  function quitarItem(productoId: number) {
    setCarrito((prev) => prev.filter((i) => i.productoId !== productoId));
  }

  const lineas = carrito.map((i) => {
    const bruto = i.cantidad * i.precioUnitario;
    const neto = bruto - i.descuentoValor;
    const impuesto = Math.round((neto * i.impuestoPct) / 100);
    return { ...i, bruto, neto, impuesto, total: neto + impuesto };
  });
  const subtotal = lineas.reduce((acc, l) => acc + l.bruto, 0);
  const descuentoLineas = lineas.reduce((acc, l) => acc + l.descuentoValor, 0);
  const impuestos = lineas.reduce((acc, l) => acc + l.impuesto, 0);
  const descuentoGlobalCentavos = Math.round(Number(descuentoGlobal || "0") * 100);
  const total = subtotal - descuentoLineas - descuentoGlobalCentavos + impuestos;
  const valorRecibidoCentavos = Math.round(Number(valorRecibido || "0") * 100);
  const cambio = metodoPago === "efectivo" ? valorRecibidoCentavos - total : 0;

  const cobrarMutacion = useMutation({
    mutationFn: () =>
      apiVentas.ventaCrear({
        clienteId: clienteId !== "general" ? Number(clienteId) : null,
        metodoPago,
        valorRecibido: metodoPago === "efectivo" ? valorRecibidoCentavos : null,
        descuentoGlobal: descuentoGlobalCentavos,
        items: lineas.map((l) => ({
          productoId: l.productoId,
          cantidad: l.cantidad,
          precioUnitario: l.precioUnitario,
          descuentoPct: 0,
          descuentoValor: l.descuentoValor,
          impuestoPct: l.impuestoPct,
        })),
      }),
    onSuccess: (venta) => {
      queryClient.invalidateQueries({ queryKey: ["productos"] });
      queryClient.invalidateQueries({ queryKey: ["ventas"] });
      toast.success(
        `Venta ${venta.venta.numeroComprobante} registrada. Total ${formatearMoneda(venta.venta.total)}` +
          (venta.venta.cambio ? ` — Cambio: ${formatearMoneda(venta.venta.cambio)}` : ""),
      );
      setCarrito([]);
      setDescuentoGlobal("0");
      setValorRecibido("");
      setClienteId("general");
      setVentaParaComprobante(venta.venta.id);
    },
    onError: (err) => toast.error(String(err)),
  });

  const puedeCobrar =
    lineas.length > 0 &&
    total >= 0 &&
    (metodoPago !== "efectivo" || valorRecibidoCentavos >= total);

  return (
    <div className="grid grid-cols-3 gap-4">
      <div className="col-span-2 space-y-4">
        <div className="relative">
          <Input
            placeholder="Buscar por nombre, SKU o código de barras…"
            value={termino}
            onChange={(e) => setTermino(e.currentTarget.value)}
          />
          {resultados && resultados.length > 0 && (
            <div className="absolute z-10 mt-1 w-full rounded-md border border-border bg-popover shadow-md">
              {resultados.map((p) => (
                <button
                  key={p.id}
                  className="flex w-full items-center justify-between px-3 py-2 text-left text-sm hover:bg-muted"
                  onClick={() => agregarAlCarrito(p)}
                >
                  <span>
                    <span className="font-mono text-xs text-muted-foreground">{p.sku}</span>{" "}
                    {p.nombre}
                  </span>
                  <span className="text-muted-foreground">{formatearMoneda(p.precioVenta)}</span>
                </button>
              ))}
            </div>
          )}
        </div>

        <div className="rounded-md border border-border">
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>Producto</TableHead>
                <TableHead className="text-right">Cant.</TableHead>
                <TableHead className="text-right">Precio</TableHead>
                <TableHead className="text-right">Desc.</TableHead>
                <TableHead className="text-right">Total</TableHead>
                <TableHead></TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {lineas.length === 0 && (
                <TableRow>
                  <TableCell colSpan={6} className="text-center text-muted-foreground">
                    Carrito vacío. Busque un producto arriba para agregarlo.
                  </TableCell>
                </TableRow>
              )}
              {lineas.map((l) => (
                <TableRow key={l.productoId}>
                  <TableCell>
                    <div className="font-medium">{l.nombre}</div>
                    <div className="font-mono text-xs text-muted-foreground">{l.sku}</div>
                  </TableCell>
                  <TableCell className="text-right">
                    <Input
                      type="number"
                      min="1"
                      step="1"
                      className="ml-auto w-16"
                      value={l.cantidad}
                      onChange={(e) =>
                        actualizarItem(l.productoId, {
                          cantidad: Math.max(1, Number(e.currentTarget.value)),
                        })
                      }
                    />
                  </TableCell>
                  <TableCell className="text-right">
                    <Input
                      type="number"
                      min="0"
                      step="0.01"
                      className="ml-auto w-24"
                      value={l.precioUnitario / 100}
                      onChange={(e) =>
                        actualizarItem(l.productoId, {
                          precioUnitario: Math.round(Number(e.currentTarget.value) * 100),
                        })
                      }
                    />
                  </TableCell>
                  <TableCell className="text-right">
                    <Input
                      type="number"
                      min="0"
                      step="0.01"
                      className="ml-auto w-20"
                      value={l.descuentoValor / 100}
                      onChange={(e) =>
                        actualizarItem(l.productoId, {
                          descuentoValor: Math.round(Number(e.currentTarget.value) * 100),
                        })
                      }
                    />
                  </TableCell>
                  <TableCell className="text-right font-medium">
                    {formatearMoneda(l.total)}
                  </TableCell>
                  <TableCell>
                    <Button variant="ghost" size="icon-sm" onClick={() => quitarItem(l.productoId)}>
                      <Trash2 className="size-4" />
                    </Button>
                  </TableCell>
                </TableRow>
              ))}
            </TableBody>
          </Table>
        </div>
      </div>

      <Card className="h-fit">
        <CardHeader>
          <CardTitle>Cobro</CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="space-y-1.5">
            <Label>Cliente</Label>
            <Select value={clienteId} onValueChange={(v) => setClienteId(v ?? "general")}>
              <SelectTrigger>
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="general">Cliente general</SelectItem>
                {clientes?.map((c) => (
                  <SelectItem key={c.id} value={String(c.id)}>
                    {c.nombre}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>

          <div className="space-y-1.5">
            <Label>Método de pago</Label>
            <Select
              value={metodoPago}
              onValueChange={(v) => setMetodoPago((v as MetodoPago) ?? "efectivo")}
            >
              <SelectTrigger>
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                {METODOS.map((m) => (
                  <SelectItem key={m.value} value={m.value}>
                    {m.label}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>

          <div className="space-y-1.5">
            <Label htmlFor="descuentoGlobal">Descuento global</Label>
            <Input
              id="descuentoGlobal"
              type="number"
              min="0"
              step="0.01"
              value={descuentoGlobal}
              onChange={(e) => setDescuentoGlobal(e.currentTarget.value)}
            />
          </div>

          <div className="space-y-1 border-t border-border pt-3 text-sm">
            <div className="flex justify-between">
              <span className="text-muted-foreground">Subtotal</span>
              <span>{formatearMoneda(subtotal)}</span>
            </div>
            <div className="flex justify-between">
              <span className="text-muted-foreground">Descuentos</span>
              <span>-{formatearMoneda(descuentoLineas + descuentoGlobalCentavos)}</span>
            </div>
            <div className="flex justify-between">
              <span className="text-muted-foreground">Impuestos</span>
              <span>{formatearMoneda(impuestos)}</span>
            </div>
            <div className="flex justify-between text-base font-semibold">
              <span>Total</span>
              <span>{formatearMoneda(Math.max(0, total))}</span>
            </div>
          </div>

          {metodoPago === "efectivo" && (
            <div className="space-y-1.5">
              <Label htmlFor="valorRecibido">Valor recibido</Label>
              <Input
                id="valorRecibido"
                type="number"
                min="0"
                step="0.01"
                value={valorRecibido}
                onChange={(e) => setValorRecibido(e.currentTarget.value)}
              />
              <p className="text-sm text-muted-foreground">
                Cambio: {formatearMoneda(Math.max(0, cambio))}
              </p>
            </div>
          )}

          <Button
            className="w-full"
            disabled={!puedeCobrar || cobrarMutacion.isPending}
            onClick={() => cobrarMutacion.mutate()}
          >
            Cobrar
          </Button>
        </CardContent>
      </Card>

      <ComprobanteDialog
        ventaId={ventaParaComprobante}
        onOpenChange={(v) => !v && setVentaParaComprobante(null)}
      />
    </div>
  );
}

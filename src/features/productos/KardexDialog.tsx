import { useState } from "react";
import { useForm, useWatch } from "react-hook-form";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { toast } from "sonner";
import { apiInventario, type Producto, type TipoMovimiento } from "@/lib/api-inventario";
import { formatearMoneda, formatearFechaHora } from "@/lib/format";
import { Dialog, DialogContent, DialogHeader, DialogTitle } from "@/components/ui/dialog";
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

interface FormValues {
  tipo: TipoMovimiento;
  cantidad: string;
  costoUnitario: string;
  motivo: string;
}

const TIPOS: { value: TipoMovimiento; label: string }[] = [
  { value: "entrada", label: "Entrada" },
  { value: "salida", label: "Salida" },
  { value: "ajuste", label: "Ajuste" },
  { value: "devolucion", label: "Devolución" },
];

const ETIQUETA_TIPO: Record<TipoMovimiento, string> = {
  entrada: "Entrada",
  salida: "Salida",
  ajuste: "Ajuste",
  devolucion: "Devolución",
};

interface Props {
  abierto: boolean;
  onOpenChange: (abierto: boolean) => void;
  producto: Producto | null;
}

export function KardexDialog({ abierto, onOpenChange, producto }: Props) {
  const queryClient = useQueryClient();
  const [mostrarForm, setMostrarForm] = useState(false);

  const { data: movimientos, isLoading } = useQuery({
    queryKey: ["kardex", producto?.id],
    queryFn: () => apiInventario.kardexGet(producto!.id),
    enabled: abierto && !!producto,
  });

  const form = useForm<FormValues>({
    defaultValues: { tipo: "entrada", cantidad: "", costoUnitario: "", motivo: "" },
  });

  const mutacion = useMutation({
    mutationFn: (values: FormValues) =>
      apiInventario.productoAjustarStock({
        productoId: producto!.id,
        tipo: values.tipo,
        cantidad: Number(values.cantidad),
        costoUnitario: values.costoUnitario ? Math.round(Number(values.costoUnitario) * 100) : null,
        motivo: values.motivo.trim() || null,
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["kardex", producto?.id] });
      queryClient.invalidateQueries({ queryKey: ["productos"] });
      toast.success("Movimiento registrado.");
      form.reset({ tipo: "entrada", cantidad: "", costoUnitario: "", motivo: "" });
      setMostrarForm(false);
    },
    onError: (err) => toast.error(String(err)),
  });

  const tipo = useWatch({ control: form.control, name: "tipo" });

  return (
    <Dialog open={abierto} onOpenChange={onOpenChange}>
      <DialogContent className="max-h-[90vh] max-w-3xl overflow-y-auto">
        <DialogHeader>
          <DialogTitle>Kardex — {producto?.nombre}</DialogTitle>
        </DialogHeader>

        <div className="flex items-center justify-between">
          <p className="text-sm text-muted-foreground">
            Stock actual:{" "}
            <span className="font-medium text-foreground">{producto?.stockActual}</span>
          </p>
          <Button size="sm" onClick={() => setMostrarForm((v) => !v)}>
            {mostrarForm ? "Cancelar" : "Ajustar stock"}
          </Button>
        </div>

        {mostrarForm && (
          <form
            onSubmit={form.handleSubmit((v) => mutacion.mutate(v))}
            className="grid grid-cols-2 gap-3 rounded-md border border-border p-3"
          >
            <div className="space-y-1.5">
              <Label>Tipo</Label>
              <Select
                value={tipo}
                onValueChange={(v) => form.setValue("tipo", v as TipoMovimiento)}
              >
                <SelectTrigger>
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  {TIPOS.map((t) => (
                    <SelectItem key={t.value} value={t.value}>
                      {t.label}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>
            <div className="space-y-1.5">
              <Label htmlFor="cantidad">
                Cantidad {tipo === "ajuste" ? "(+ agrega, - quita)" : ""}
              </Label>
              <Input id="cantidad" type="number" step="1" {...form.register("cantidad")} />
            </div>
            {tipo === "entrada" && (
              <div className="space-y-1.5">
                <Label htmlFor="costoUnitario">Costo unitario (opcional)</Label>
                <Input
                  id="costoUnitario"
                  type="number"
                  step="0.01"
                  min="0"
                  {...form.register("costoUnitario")}
                />
              </div>
            )}
            <div
              className={tipo === "entrada" ? "col-span-2 space-y-1.5" : "col-span-2 space-y-1.5"}
            >
              <Label htmlFor="motivo">
                Motivo {tipo === "ajuste" ? "(obligatorio)" : "(opcional)"}
              </Label>
              <Input id="motivo" {...form.register("motivo")} />
            </div>
            <div className="col-span-2">
              <Button type="submit" disabled={mutacion.isPending}>
                Registrar movimiento
              </Button>
            </div>
          </form>
        )}

        <div className="max-h-96 overflow-y-auto rounded-md border border-border">
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>Fecha</TableHead>
                <TableHead>Tipo</TableHead>
                <TableHead className="text-right">Cantidad</TableHead>
                <TableHead className="text-right">Costo unit.</TableHead>
                <TableHead className="text-right">Saldo</TableHead>
                <TableHead>Motivo</TableHead>
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
              {!isLoading && movimientos?.length === 0 && (
                <TableRow>
                  <TableCell colSpan={6} className="text-center text-muted-foreground">
                    Sin movimientos todavía.
                  </TableCell>
                </TableRow>
              )}
              {movimientos?.map((m) => (
                <TableRow key={m.id}>
                  <TableCell className="whitespace-nowrap text-sm">
                    {formatearFechaHora(m.fecha)}
                  </TableCell>
                  <TableCell>{ETIQUETA_TIPO[m.tipo]}</TableCell>
                  <TableCell className="text-right">{m.cantidad}</TableCell>
                  <TableCell className="text-right">
                    {m.costoUnitario != null ? formatearMoneda(m.costoUnitario) : "—"}
                  </TableCell>
                  <TableCell className="text-right font-medium">{m.saldoResultante}</TableCell>
                  <TableCell className="text-sm text-muted-foreground">{m.motivo ?? "—"}</TableCell>
                </TableRow>
              ))}
            </TableBody>
          </Table>
        </div>
      </DialogContent>
    </Dialog>
  );
}

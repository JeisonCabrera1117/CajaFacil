import { useState } from "react";
import { useMutation } from "@tanstack/react-query";
import { toast } from "sonner";
import {
  apiComprobantes,
  ETIQUETA_TIPO_COMPROBANTE,
  type Comprobante,
  type TipoComprobante,
} from "@/lib/api-comprobantes";
import { Button } from "@/components/ui/button";
import { Label } from "@/components/ui/label";
import {
  Dialog,
  DialogContent,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";

interface Props {
  ventaId: number | null;
  onOpenChange: (abierto: boolean) => void;
  /** "regenerar" se usa desde el historial (la venta puede ya tener comprobantes previos). */
  modo?: "nuevo" | "regenerar";
}

export function ComprobanteDialog({ ventaId, onOpenChange, modo = "nuevo" }: Props) {
  const [tipo, setTipo] = useState<TipoComprobante>("termico80");
  const [generado, setGenerado] = useState<Comprobante | null>(null);

  const generarMutacion = useMutation({
    mutationFn: () => {
      const fn = modo === "regenerar" ? apiComprobantes.regenerar : apiComprobantes.generar;
      return fn(ventaId!, tipo);
    },
    onSuccess: (comprobante) => {
      setGenerado(comprobante);
      toast.success("Comprobante generado.");
    },
    onError: (err) => toast.error(String(err)),
  });

  const abrirMutacion = useMutation({
    mutationFn: () => apiComprobantes.abrir(generado!.rutaArchivo),
    onError: (err) => toast.error(String(err)),
  });

  const copiarMutacion = useMutation({
    mutationFn: () => apiComprobantes.copiarImagen(generado!.rutaArchivo),
    onSuccess: () => toast.success("Imagen copiada al portapapeles."),
    onError: (err) => toast.error(String(err)),
  });

  function cerrar(abierto: boolean) {
    if (!abierto) {
      setGenerado(null);
      generarMutacion.reset();
    }
    onOpenChange(abierto);
  }

  return (
    <Dialog open={ventaId != null} onOpenChange={cerrar}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>Comprobante de venta</DialogTitle>
        </DialogHeader>

        <div className="space-y-1.5">
          <Label>Formato</Label>
          <Select
            value={tipo}
            onValueChange={(v) => setTipo((v as TipoComprobante) ?? "termico80")}
          >
            <SelectTrigger>
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              {Object.entries(ETIQUETA_TIPO_COMPROBANTE).map(([valor, etiqueta]) => (
                <SelectItem key={valor} value={valor}>
                  {etiqueta}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </div>

        {generado && (
          <p className="break-all rounded-md bg-muted p-2 text-xs text-muted-foreground">
            {generado.rutaArchivo}
          </p>
        )}

        <DialogFooter className="flex-wrap gap-2">
          <Button variant="outline" onClick={() => cerrar(false)}>
            Cerrar
          </Button>
          {generado && generado.tipo === "imagen" && (
            <Button
              variant="outline"
              disabled={copiarMutacion.isPending}
              onClick={() => copiarMutacion.mutate()}
            >
              Copiar imagen
            </Button>
          )}
          {generado && (
            <Button
              variant="outline"
              disabled={abrirMutacion.isPending}
              onClick={() => abrirMutacion.mutate()}
            >
              Abrir / Imprimir
            </Button>
          )}
          <Button disabled={generarMutacion.isPending} onClick={() => generarMutacion.mutate()}>
            {generado ? "Generar de nuevo" : "Generar"}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}

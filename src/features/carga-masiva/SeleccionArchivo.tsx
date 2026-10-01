import { useEffect, useState } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useMutation, useQuery } from "@tanstack/react-query";
import { toast } from "sonner";
import { apiImportacion, descargarPlantilla, type DeteccionArchivo } from "@/lib/api-importacion";
import { Button } from "@/components/ui/button";
import { Label } from "@/components/ui/label";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";

interface Props {
  entidad: string;
  onEntidadChange: (entidad: string) => void;
  ruta: string | null;
  onArchivo: (ruta: string, nombreOriginal: string) => void;
  deteccion: DeteccionArchivo | null;
  onDeteccionCambiada: (d: DeteccionArchivo) => void;
}

function nombreDesdeRuta(ruta: string): string {
  return ruta.split(/[\\/]/).pop() ?? ruta;
}

export function SeleccionArchivo({
  entidad,
  onEntidadChange,
  ruta,
  onArchivo,
  deteccion,
  onDeteccionCambiada,
}: Props) {
  const [arrastrando, setArrastrando] = useState(false);

  const { data: entidades } = useQuery({
    queryKey: ["import-entidades"],
    queryFn: apiImportacion.entidades,
  });

  useEffect(() => {
    let cancelado = false;
    const promesa = getCurrentWindow().onDragDropEvent((evento) => {
      if (evento.payload.type === "over") setArrastrando(true);
      if (evento.payload.type === "leave") setArrastrando(false);
      if (evento.payload.type === "drop") {
        setArrastrando(false);
        const primero = evento.payload.paths[0];
        if (primero) onArchivo(primero, nombreDesdeRuta(primero));
      }
    });
    return () => {
      if (!cancelado) promesa.then((unlisten) => unlisten());
      cancelado = true;
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  const detectarMutacion = useMutation({
    mutationFn: (opciones: { hoja?: string; separador?: string }) =>
      apiImportacion.detectarArchivo(ruta!, opciones.hoja, opciones.separador),
    onSuccess: onDeteccionCambiada,
    onError: (err) => toast.error(String(err)),
  });

  const plantillaMutacion = useMutation({
    mutationFn: (formato: "csv" | "xlsx") => apiImportacion.plantillaDescargar(entidad, formato),
    onSuccess: descargarPlantilla,
    onError: (err) => toast.error(String(err)),
  });

  async function elegirArchivo() {
    const seleccionado = await open({
      multiple: false,
      filters: [{ name: "Datos", extensions: ["csv", "xlsx", "xls"] }],
    });
    if (typeof seleccionado === "string") {
      onArchivo(seleccionado, nombreDesdeRuta(seleccionado));
    }
  }

  useEffect(() => {
    if (ruta) detectarMutacion.mutate({});
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [ruta]);

  return (
    <div className="space-y-4">
      <div className="space-y-1.5">
        <Label>Qué vas a importar</Label>
        <Select value={entidad} onValueChange={(v) => v && onEntidadChange(v)}>
          <SelectTrigger className="max-w-sm">
            <SelectValue placeholder="Elegir entidad…" />
          </SelectTrigger>
          <SelectContent>
            {entidades?.map((e) => (
              <SelectItem key={e.id} value={e.id}>
                {e.etiqueta}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
      </div>

      {entidad && (
        <div className="flex gap-2">
          <Button variant="outline" size="sm" onClick={() => plantillaMutacion.mutate("csv")}>
            Descargar plantilla CSV
          </Button>
          <Button variant="outline" size="sm" onClick={() => plantillaMutacion.mutate("xlsx")}>
            Descargar plantilla XLSX
          </Button>
        </div>
      )}

      {entidad && (
        <div
          className={`rounded-md border-2 border-dashed p-6 text-center transition-colors ${
            arrastrando ? "border-primary bg-muted" : "border-border"
          }`}
        >
          <p className="text-sm text-muted-foreground">Arrastrá acá tu archivo CSV o XLSX, o</p>
          <Button className="mt-2" onClick={elegirArchivo}>
            Elegir archivo…
          </Button>
          {ruta && (
            <p className="mt-2 break-all text-xs text-muted-foreground">{nombreDesdeRuta(ruta)}</p>
          )}
        </div>
      )}

      {deteccion && (
        <div className="grid grid-cols-2 gap-4 rounded-md border border-border p-3 sm:grid-cols-4">
          <div>
            <Label className="text-xs text-muted-foreground">Tipo</Label>
            <p className="text-sm uppercase">{deteccion.tipo}</p>
          </div>
          {deteccion.tipo === "csv" && (
            <>
              <div>
                <Label className="text-xs text-muted-foreground">Codificación</Label>
                <p className="text-sm">{deteccion.encoding}</p>
              </div>
              <div className="space-y-1">
                <Label className="text-xs text-muted-foreground">Separador</Label>
                <Select
                  value={deteccion.separador ?? ","}
                  onValueChange={(v) => v && detectarMutacion.mutate({ separador: v })}
                >
                  <SelectTrigger className="h-8">
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value=",">Coma (,)</SelectItem>
                    <SelectItem value=";">Punto y coma (;)</SelectItem>
                  </SelectContent>
                </Select>
              </div>
            </>
          )}
          {deteccion.tipo === "xlsx" && deteccion.hojas.length > 1 && (
            <div className="space-y-1">
              <Label className="text-xs text-muted-foreground">Hoja</Label>
              <Select
                value={deteccion.hojaSeleccionada ?? deteccion.hojas[0]}
                onValueChange={(v) => v && detectarMutacion.mutate({ hoja: v })}
              >
                <SelectTrigger className="h-8">
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  {deteccion.hojas.map((h) => (
                    <SelectItem key={h} value={h}>
                      {h}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>
          )}
          <div>
            <Label className="text-xs text-muted-foreground">Filas detectadas</Label>
            <p className="text-sm">{deteccion.totalFilas}</p>
          </div>
        </div>
      )}
    </div>
  );
}

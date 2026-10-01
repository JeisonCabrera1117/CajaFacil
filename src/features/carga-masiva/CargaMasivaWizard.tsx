import { useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { toast } from "sonner";
import {
  apiImportacion,
  type DeteccionArchivo,
  type ModoImport,
  type ProgresoImport,
  type ResumenImportacion,
} from "@/lib/api-importacion";
import { sugerirMapeo } from "@/features/carga-masiva/mapeo";
import { SeleccionArchivo } from "@/features/carga-masiva/SeleccionArchivo";
import { MapeoColumnas } from "@/features/carga-masiva/MapeoColumnas";
import { ResumenPanel } from "@/features/carga-masiva/ResumenPanel";
import { Button } from "@/components/ui/button";
import { Label } from "@/components/ui/label";
import { Progress } from "@/components/ui/progress";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";

const MODOS: { value: ModoImport; label: string; descripcion: string }[] = [
  { value: "crear", label: "Solo crear", descripcion: "Omite las filas que ya existan." },
  {
    value: "actualizar",
    label: "Solo actualizar",
    descripcion: "Omite las filas que no existan todavía.",
  },
  {
    value: "crear_y_actualizar",
    label: "Crear y actualizar",
    descripcion: "Crea lo nuevo y actualiza lo existente.",
  },
];

export function CargaMasivaWizard() {
  const queryClient = useQueryClient();
  const [entidad, setEntidad] = useState("");
  const [ruta, setRuta] = useState<string | null>(null);
  const [nombreOriginal, setNombreOriginal] = useState<string | null>(null);
  const [deteccion, setDeteccion] = useState<DeteccionArchivo | null>(null);
  const [mapeo, setMapeo] = useState<Record<string, string>>({});
  const [modo, setModo] = useState<ModoImport>("crear");
  const [resumenPreview, setResumenPreview] = useState<ResumenImportacion | null>(null);
  const [progreso, setProgreso] = useState<ProgresoImport | null>(null);

  const { data: campos } = useQuery({
    queryKey: ["import-campos", entidad],
    queryFn: () => apiImportacion.campos(entidad),
    enabled: !!entidad,
  });

  function reiniciarArchivo() {
    setRuta(null);
    setNombreOriginal(null);
    setDeteccion(null);
    setMapeo({});
    setResumenPreview(null);
  }

  function cambiarEntidad(nueva: string) {
    setEntidad(nueva);
    reiniciarArchivo();
  }

  function onArchivo(rutaElegida: string, nombre: string) {
    setRuta(rutaElegida);
    setNombreOriginal(nombre);
    setResumenPreview(null);
  }

  useEffect(() => {
    // Sincroniza la sugerencia automática de mapeo cuando cambian las columnas
    // detectadas; es estado editable independiente (el usuario puede corregir
    // la sugerencia antes de confirmar), no algo derivable en el render.
    if (campos && deteccion) {
      // eslint-disable-next-line react-hooks/set-state-in-effect
      setMapeo((previo) => {
        const sugerido = sugerirMapeo(campos, deteccion.columnas);
        return { ...sugerido, ...previo };
      });
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [campos, deteccion?.columnas.join("|")]);

  const mapeoUtil = Object.fromEntries(Object.entries(mapeo).filter(([, v]) => v));
  const faltanObligatorios = (campos ?? []).some((c) => c.obligatorio && !mapeoUtil[c.id]);

  const previewMutacion = useMutation({
    mutationFn: () =>
      apiImportacion.preview({
        ruta: ruta!,
        hoja: deteccion?.hojaSeleccionada ?? null,
        separador: deteccion?.separador ?? null,
        entidad,
        mapeo: mapeoUtil,
        modo,
      }),
    onSuccess: setResumenPreview,
    onError: (err) => toast.error(String(err)),
  });

  const ejecutarMutacion = useMutation({
    mutationFn: async () => {
      setProgreso({ procesadas: 0, total: deteccion?.totalFilas ?? 0 });
      const unlisten = await listen<ProgresoImport>("import-progreso", (evento) =>
        setProgreso(evento.payload),
      );
      try {
        return await apiImportacion.ejecutar({
          ruta: ruta!,
          hoja: deteccion?.hojaSeleccionada ?? null,
          separador: deteccion?.separador ?? null,
          entidad,
          mapeo: mapeoUtil,
          modo,
          nombreArchivoOriginal: nombreOriginal ?? "archivo",
        });
      } finally {
        unlisten();
      }
    },
    onSuccess: (resultado) => {
      toast.success(
        `Importación completa: ${resultado.creados} creados, ${resultado.actualizados} actualizados.`,
      );
      queryClient.invalidateQueries({ queryKey: ["import-historial"] });
      queryClient.invalidateQueries({ queryKey: ["productos"] });
      queryClient.invalidateQueries({ queryKey: ["categorias"] });
      queryClient.invalidateQueries({ queryKey: ["proveedores"] });
      queryClient.invalidateQueries({ queryKey: ["clientes"] });
      queryClient.invalidateQueries({ queryKey: ["ventas"] });
      setProgreso(null);
    },
    onError: (err) => {
      toast.error(String(err));
      setProgreso(null);
    },
  });

  const listoParaPreview = !!ruta && !!entidad && !faltanObligatorios;

  return (
    <div className="space-y-6">
      <Card>
        <CardHeader>
          <CardTitle>1. Entidad y archivo</CardTitle>
        </CardHeader>
        <CardContent>
          <SeleccionArchivo
            entidad={entidad}
            onEntidadChange={cambiarEntidad}
            ruta={ruta}
            onArchivo={onArchivo}
            deteccion={deteccion}
            onDeteccionCambiada={setDeteccion}
          />
        </CardContent>
      </Card>

      {deteccion && campos && campos.length > 0 && (
        <Card>
          <CardHeader>
            <CardTitle>2. Mapeo de columnas y modo</CardTitle>
          </CardHeader>
          <CardContent className="space-y-4">
            <MapeoColumnas
              campos={campos}
              columnas={deteccion.columnas}
              mapeo={mapeo}
              onCambiar={(campoId, columna) =>
                setMapeo((prev) => ({ ...prev, [campoId]: columna }))
              }
            />

            <div className="max-w-sm space-y-1.5">
              <Label>Modo de importación</Label>
              <Select value={modo} onValueChange={(v) => v && setModo(v as ModoImport)}>
                <SelectTrigger>
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  {MODOS.map((m) => (
                    <SelectItem key={m.value} value={m.value}>
                      {m.label}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
              <p className="text-xs text-muted-foreground">
                {MODOS.find((m) => m.value === modo)?.descripcion}
              </p>
            </div>

            <Button
              disabled={!listoParaPreview || previewMutacion.isPending}
              onClick={() => previewMutacion.mutate()}
            >
              Vista previa
            </Button>
          </CardContent>
        </Card>
      )}

      {resumenPreview && (
        <Card>
          <CardHeader>
            <CardTitle>3. Vista previa</CardTitle>
          </CardHeader>
          <CardContent className="space-y-4">
            <ResumenPanel resumen={resumenPreview} />
            <Button disabled={ejecutarMutacion.isPending} onClick={() => ejecutarMutacion.mutate()}>
              Ejecutar importación
            </Button>
          </CardContent>
        </Card>
      )}

      {progreso && (
        <Card>
          <CardContent className="space-y-2 pt-6">
            <Progress
              value={progreso.total > 0 ? (progreso.procesadas / progreso.total) * 100 : 0}
            />
            <p className="text-center text-sm text-muted-foreground">
              {progreso.procesadas} / {progreso.total} filas procesadas
            </p>
          </CardContent>
        </Card>
      )}

      {ejecutarMutacion.data && (
        <Card>
          <CardHeader>
            <CardTitle>Resultado</CardTitle>
          </CardHeader>
          <CardContent>
            <ResumenPanel
              resumen={{
                totalFilas: ejecutarMutacion.data.totalFilas,
                creados: ejecutarMutacion.data.creados,
                actualizados: ejecutarMutacion.data.actualizados,
                omitidos: ejecutarMutacion.data.omitidos,
                conError: ejecutarMutacion.data.conError,
                filasRechazadas: resumenPreview?.filasRechazadas ?? [],
              }}
            />
          </CardContent>
        </Card>
      )}
    </div>
  );
}

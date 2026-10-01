import { useMutation } from "@tanstack/react-query";
import { toast } from "sonner";
import { apiReportes, type FormatoExportacion } from "@/lib/api-reportes";
import { descargarPlantilla } from "@/lib/api-importacion";
import { Button } from "@/components/ui/button";

interface Props {
  titulo: string;
  columnas: string[];
  filas: string[][];
}

const FORMATOS: { valor: FormatoExportacion; etiqueta: string }[] = [
  { valor: "csv", etiqueta: "CSV" },
  { valor: "xlsx", etiqueta: "XLSX" },
  { valor: "pdf", etiqueta: "PDF" },
];

export function ExportarReporteBotones({ titulo, columnas, filas }: Props) {
  const exportarMutacion = useMutation({
    mutationFn: (formato: FormatoExportacion) =>
      apiReportes.exportar(titulo, columnas, filas, formato),
    onSuccess: descargarPlantilla,
    onError: (err) => toast.error(String(err)),
  });

  return (
    <div className="flex gap-2">
      {FORMATOS.map((f) => (
        <Button
          key={f.valor}
          variant="outline"
          size="sm"
          disabled={filas.length === 0 || exportarMutacion.isPending}
          onClick={() => exportarMutacion.mutate(f.valor)}
        >
          Exportar {f.etiqueta}
        </Button>
      ))}
    </div>
  );
}

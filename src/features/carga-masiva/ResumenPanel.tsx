import type { ResumenImportacion } from "@/lib/api-importacion";
import { Badge } from "@/components/ui/badge";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";

export function ResumenPanel({ resumen }: { resumen: ResumenImportacion }) {
  return (
    <div className="space-y-3">
      <div className="grid grid-cols-2 gap-3 sm:grid-cols-5">
        <Estadistica etiqueta="Filas" valor={resumen.totalFilas} />
        <Estadistica etiqueta="Creados" valor={resumen.creados} variante="success" />
        <Estadistica etiqueta="Actualizados" valor={resumen.actualizados} variante="secondary" />
        <Estadistica etiqueta="Omitidos" valor={resumen.omitidos} variante="warning" />
        <Estadistica etiqueta="Con error" valor={resumen.conError} variante="destructive" />
      </div>

      {resumen.filasRechazadas.length > 0 && (
        <div className="max-h-64 overflow-y-auto rounded-md border border-border">
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead className="w-20">Fila</TableHead>
                <TableHead>Motivo</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {resumen.filasRechazadas.map((f, i) => (
                <TableRow key={i}>
                  <TableCell>{f.fila}</TableCell>
                  <TableCell className="text-sm text-muted-foreground">{f.motivo}</TableCell>
                </TableRow>
              ))}
            </TableBody>
          </Table>
        </div>
      )}
    </div>
  );
}

function Estadistica({
  etiqueta,
  valor,
  variante,
}: {
  etiqueta: string;
  valor: number;
  variante?: "default" | "secondary" | "destructive" | "success" | "warning";
}) {
  return (
    <div className="rounded-md border border-border p-3 text-center">
      <p className="text-2xl font-semibold">{valor}</p>
      <Badge variant={variante ?? "outline"} className="mt-1">
        {etiqueta}
      </Badge>
    </div>
  );
}

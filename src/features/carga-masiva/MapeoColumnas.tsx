import type { CampoImport } from "@/lib/api-importacion";
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

interface Props {
  campos: CampoImport[];
  columnas: string[];
  mapeo: Record<string, string>;
  onCambiar: (campoId: string, columna: string) => void;
}

export function MapeoColumnas({ campos, columnas, mapeo, onCambiar }: Props) {
  return (
    <div className="space-y-2">
      <Label>Relacioná cada campo con la columna de tu archivo</Label>
      <div className="rounded-md border border-border">
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead>Campo del sistema</TableHead>
              <TableHead>Columna del archivo</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {campos.map((campo) => (
              <TableRow key={campo.id}>
                <TableCell>
                  {campo.etiqueta}
                  {campo.obligatorio && <span className="ml-1 text-destructive">*</span>}
                </TableCell>
                <TableCell>
                  <Select
                    value={mapeo[campo.id] ?? "none"}
                    onValueChange={(v) => onCambiar(campo.id, v === "none" ? "" : (v ?? ""))}
                  >
                    <SelectTrigger className="w-full max-w-xs">
                      <SelectValue placeholder="Sin asignar" />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="none">Sin asignar</SelectItem>
                      {columnas.map((c) => (
                        <SelectItem key={c} value={c}>
                          {c}
                        </SelectItem>
                      ))}
                    </SelectContent>
                  </Select>
                </TableCell>
              </TableRow>
            ))}
          </TableBody>
        </Table>
      </div>
    </div>
  );
}

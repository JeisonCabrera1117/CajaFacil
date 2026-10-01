import { useState } from "react";
import { useQuery } from "@tanstack/react-query";
import { apiVentas, type Cliente } from "@/lib/api-ventas";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import { ClienteFormDialog } from "@/features/clientes/ClienteFormDialog";

export function ClientesPanel() {
  const [busqueda, setBusqueda] = useState("");
  const [formAbierto, setFormAbierto] = useState(false);
  const [seleccionado, setSeleccionado] = useState<Cliente | null>(null);

  const { data: clientes, isLoading } = useQuery({
    queryKey: ["clientes"],
    queryFn: apiVentas.clienteList,
  });

  const filtrados = clientes?.filter((c) => {
    const termino = busqueda.trim().toLowerCase();
    if (!termino) return true;
    return (
      c.nombre.toLowerCase().includes(termino) ||
      (c.documento ?? "").toLowerCase().includes(termino) ||
      (c.telefono ?? "").toLowerCase().includes(termino)
    );
  });

  return (
    <div className="space-y-4">
      <div className="flex items-center gap-3">
        <Input
          placeholder="Buscar por nombre, documento o teléfono…"
          value={busqueda}
          onChange={(e) => setBusqueda(e.currentTarget.value)}
          className="max-w-xs"
        />
        <div className="flex-1" />
        <Button
          onClick={() => {
            setSeleccionado(null);
            setFormAbierto(true);
          }}
        >
          Nuevo cliente
        </Button>
      </div>

      <div className="rounded-md border border-border">
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead>Nombre</TableHead>
              <TableHead>Documento</TableHead>
              <TableHead>Teléfono</TableHead>
              <TableHead>Correo</TableHead>
              <TableHead className="text-right">Acciones</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {isLoading && (
              <TableRow>
                <TableCell colSpan={5} className="text-center text-muted-foreground">
                  Cargando…
                </TableCell>
              </TableRow>
            )}
            {!isLoading && filtrados?.length === 0 && (
              <TableRow>
                <TableCell colSpan={5} className="text-center text-muted-foreground">
                  No hay clientes que coincidan.
                </TableCell>
              </TableRow>
            )}
            {filtrados?.map((c) => (
              <TableRow key={c.id}>
                <TableCell>{c.nombre}</TableCell>
                <TableCell className="text-muted-foreground">{c.documento ?? "—"}</TableCell>
                <TableCell className="text-muted-foreground">{c.telefono ?? "—"}</TableCell>
                <TableCell className="text-muted-foreground">{c.correo ?? "—"}</TableCell>
                <TableCell className="text-right">
                  <Button
                    variant="outline"
                    size="sm"
                    onClick={() => {
                      setSeleccionado(c);
                      setFormAbierto(true);
                    }}
                  >
                    Editar
                  </Button>
                </TableCell>
              </TableRow>
            ))}
          </TableBody>
        </Table>
      </div>

      <ClienteFormDialog
        abierto={formAbierto}
        onOpenChange={setFormAbierto}
        cliente={seleccionado}
      />
    </div>
  );
}

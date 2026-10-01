import { ClientesPanel } from "@/features/clientes/ClientesPanel";

export function Clientes() {
  return (
    <div className="space-y-4">
      <h1 className="text-2xl font-semibold">Clientes</h1>
      <ClientesPanel />
    </div>
  );
}

import { ProveedoresPanel } from "@/features/proveedores/ProveedoresPanel";

export function Proveedores() {
  return (
    <div className="space-y-4">
      <h1 className="text-2xl font-semibold">Proveedores</h1>
      <ProveedoresPanel />
    </div>
  );
}

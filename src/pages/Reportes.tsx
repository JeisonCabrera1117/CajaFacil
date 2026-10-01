import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { ReporteVentasPanel } from "@/features/reportes/ReporteVentasPanel";
import { ReporteUtilidadPanel } from "@/features/reportes/ReporteUtilidadPanel";
import { ReporteInventarioPanel } from "@/features/reportes/ReporteInventarioPanel";
import { ReporteRotacionPanel } from "@/features/reportes/ReporteRotacionPanel";
import { ReporteComprasProveedorPanel } from "@/features/reportes/ReporteComprasProveedorPanel";

export function Reportes() {
  return (
    <div className="space-y-4">
      <h1 className="text-2xl font-semibold">Reportes</h1>

      <Tabs defaultValue="ventas">
        <TabsList>
          <TabsTrigger value="ventas">Ventas</TabsTrigger>
          <TabsTrigger value="utilidad">Utilidad</TabsTrigger>
          <TabsTrigger value="inventario">Inventario valorizado</TabsTrigger>
          <TabsTrigger value="rotacion">Rotación</TabsTrigger>
          <TabsTrigger value="compras">Compras por proveedor</TabsTrigger>
        </TabsList>
        <TabsContent value="ventas">
          <ReporteVentasPanel />
        </TabsContent>
        <TabsContent value="utilidad">
          <ReporteUtilidadPanel />
        </TabsContent>
        <TabsContent value="inventario">
          <ReporteInventarioPanel />
        </TabsContent>
        <TabsContent value="rotacion">
          <ReporteRotacionPanel />
        </TabsContent>
        <TabsContent value="compras">
          <ReporteComprasProveedorPanel />
        </TabsContent>
      </Tabs>
    </div>
  );
}

import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { CajaWidget } from "@/features/caja/CajaWidget";
import { Pos } from "@/features/ventas/Pos";
import { HistorialVentasPanel } from "@/features/ventas/HistorialVentasPanel";

export function Ventas() {
  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <h1 className="text-2xl font-semibold">Ventas</h1>
      </div>

      <CajaWidget />

      <Tabs defaultValue="pos">
        <TabsList>
          <TabsTrigger value="pos">Punto de venta</TabsTrigger>
          <TabsTrigger value="historial">Historial</TabsTrigger>
        </TabsList>
        <TabsContent value="pos">
          <Pos />
        </TabsContent>
        <TabsContent value="historial">
          <HistorialVentasPanel />
        </TabsContent>
      </Tabs>
    </div>
  );
}

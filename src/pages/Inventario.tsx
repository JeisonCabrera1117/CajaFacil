import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { ProductosPanel } from "@/features/productos/ProductosPanel";
import { CategoriasPanel } from "@/features/categorias/CategoriasPanel";
import { ComprasPanel } from "@/features/compras/ComprasPanel";
import { InventarioFisicoPanel } from "@/features/inventario-fisico/InventarioFisicoPanel";

export function Inventario() {
  return (
    <div className="space-y-4">
      <h1 className="text-2xl font-semibold">Inventario</h1>

      <Tabs defaultValue="productos">
        <TabsList>
          <TabsTrigger value="productos">Productos</TabsTrigger>
          <TabsTrigger value="categorias">Categorías</TabsTrigger>
          <TabsTrigger value="compras">Compras</TabsTrigger>
          <TabsTrigger value="inventario-fisico">Toma de inventario</TabsTrigger>
        </TabsList>
        <TabsContent value="productos">
          <ProductosPanel />
        </TabsContent>
        <TabsContent value="categorias">
          <CategoriasPanel />
        </TabsContent>
        <TabsContent value="compras">
          <ComprasPanel />
        </TabsContent>
        <TabsContent value="inventario-fisico">
          <InventarioFisicoPanel />
        </TabsContent>
      </Tabs>
    </div>
  );
}

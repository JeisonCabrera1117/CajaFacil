import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { CargaMasivaWizard } from "@/features/carga-masiva/CargaMasivaWizard";
import { HistorialImportacionesPanel } from "@/features/carga-masiva/HistorialImportacionesPanel";

export function CargaMasiva() {
  return (
    <div className="space-y-4">
      <h1 className="text-2xl font-semibold">Carga masiva</h1>

      <Tabs defaultValue="nueva">
        <TabsList>
          <TabsTrigger value="nueva">Nueva importación</TabsTrigger>
          <TabsTrigger value="historial">Historial</TabsTrigger>
        </TabsList>
        <TabsContent value="nueva">
          <CargaMasivaWizard />
        </TabsContent>
        <TabsContent value="historial">
          <HistorialImportacionesPanel />
        </TabsContent>
      </Tabs>
    </div>
  );
}

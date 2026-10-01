import { useEffect } from "react";
import { useForm, useWatch } from "react-hook-form";
import { zodResolver } from "@hookform/resolvers/zod";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import { toast } from "sonner";
import { api, type EmpresaConfig } from "@/lib/tauri";
import { empresaConfigSchema, type EmpresaConfigFormValues } from "@/features/config/schema";
import { SeccionPin } from "@/features/config/SeccionPin";
import { RespaldosPanel } from "@/features/config/RespaldosPanel";
import { AuditoriaPanel } from "@/features/config/AuditoriaPanel";
import { ActualizacionesPanel } from "@/features/config/ActualizacionesPanel";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Switch } from "@/components/ui/switch";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";

function aFormulario(config: EmpresaConfig): EmpresaConfigFormValues {
  return {
    nombre: config.nombre,
    nit: config.nit,
    direccion: config.direccion,
    telefono: config.telefono,
    logoPath: config.logoPath ?? "",
    moneda: config.moneda,
    formatoFecha: config.formatoFecha as EmpresaConfigFormValues["formatoFecha"],
    prefijoComprobante: config.prefijoComprobante,
    leyendaPie: config.leyendaPie,
    tema: config.tema as EmpresaConfigFormValues["tema"],
    permiteStockNegativo: config.permiteStockNegativo,
    arqueoActivo: config.arqueoActivo,
  };
}

export function Configuracion() {
  const queryClient = useQueryClient();
  const { data: config, isLoading } = useQuery({ queryKey: ["config"], queryFn: api.configGet });

  const form = useForm<EmpresaConfigFormValues>({
    resolver: zodResolver(empresaConfigSchema),
    defaultValues: {
      nombre: "",
      nit: "",
      direccion: "",
      telefono: "",
      logoPath: "",
      moneda: "COP",
      formatoFecha: "DD/MM/YYYY",
      prefijoComprobante: "CF",
      leyendaPie: "Este documento no es una factura electrónica",
      tema: "claro",
      permiteStockNegativo: false,
      arqueoActivo: false,
    },
  });

  useEffect(() => {
    if (config) form.reset(aFormulario(config));
  }, [config, form]);

  const formatoFecha = useWatch({ control: form.control, name: "formatoFecha" });
  const tema = useWatch({ control: form.control, name: "tema" });
  const permiteStockNegativo = useWatch({ control: form.control, name: "permiteStockNegativo" });
  const arqueoActivo = useWatch({ control: form.control, name: "arqueoActivo" });

  async function onSubmit(values: EmpresaConfigFormValues) {
    try {
      const actualizado = await api.configUpdate({
        ...values,
        logoPath: values.logoPath.trim() === "" ? null : values.logoPath.trim(),
      });
      queryClient.setQueryData(["config"], actualizado);
      toast.success("Configuración guardada.");
    } catch (err) {
      toast.error(String(err));
    }
  }

  if (isLoading || !config) {
    return <p className="text-muted-foreground">Cargando…</p>;
  }

  return (
    <div className="max-w-3xl space-y-6">
      <h1 className="text-2xl font-semibold">Configuración</h1>

      <Tabs defaultValue="general">
        <TabsList>
          <TabsTrigger value="general">General</TabsTrigger>
          <TabsTrigger value="respaldos">Respaldos</TabsTrigger>
          <TabsTrigger value="actualizaciones">Actualizaciones</TabsTrigger>
          <TabsTrigger value="historial">Historial de cambios</TabsTrigger>
        </TabsList>

        <TabsContent value="general" className="space-y-6">
          <Card>
            <CardHeader>
              <CardTitle>Datos de la empresa</CardTitle>
            </CardHeader>
            <CardContent>
              <form onSubmit={form.handleSubmit(onSubmit)} className="space-y-4">
                <div className="grid grid-cols-2 gap-4">
                  <div className="space-y-1.5">
                    <Label htmlFor="nombre">Nombre</Label>
                    <Input id="nombre" {...form.register("nombre")} />
                    {form.formState.errors.nombre && (
                      <p className="text-sm text-destructive">
                        {form.formState.errors.nombre.message}
                      </p>
                    )}
                  </div>
                  <div className="space-y-1.5">
                    <Label htmlFor="nit">NIT</Label>
                    <Input id="nit" {...form.register("nit")} />
                  </div>
                  <div className="space-y-1.5">
                    <Label htmlFor="telefono">Teléfono</Label>
                    <Input id="telefono" {...form.register("telefono")} />
                  </div>
                  <div className="col-span-2 space-y-1.5">
                    <Label htmlFor="direccion">Dirección</Label>
                    <Input id="direccion" {...form.register("direccion")} />
                  </div>
                  <div className="col-span-2 space-y-1.5">
                    <Label htmlFor="logoPath">Ruta del logo (opcional)</Label>
                    <Input
                      id="logoPath"
                      placeholder="C:\ruta\logo.png"
                      {...form.register("logoPath")}
                    />
                  </div>
                </div>

                <div className="grid grid-cols-3 gap-4">
                  <div className="space-y-1.5">
                    <Label htmlFor="moneda">Moneda</Label>
                    <Input id="moneda" {...form.register("moneda")} />
                  </div>
                  <div className="space-y-1.5">
                    <Label>Formato de fecha</Label>
                    <Select
                      value={formatoFecha}
                      onValueChange={(v) =>
                        form.setValue("formatoFecha", v as EmpresaConfigFormValues["formatoFecha"])
                      }
                    >
                      <SelectTrigger>
                        <SelectValue />
                      </SelectTrigger>
                      <SelectContent>
                        <SelectItem value="DD/MM/YYYY">DD/MM/AAAA</SelectItem>
                        <SelectItem value="MM/DD/YYYY">MM/DD/AAAA</SelectItem>
                        <SelectItem value="YYYY-MM-DD">AAAA-MM-DD</SelectItem>
                      </SelectContent>
                    </Select>
                  </div>
                  <div className="space-y-1.5">
                    <Label htmlFor="prefijoComprobante">Prefijo comprobante</Label>
                    <Input id="prefijoComprobante" {...form.register("prefijoComprobante")} />
                  </div>
                </div>

                <div className="space-y-1.5">
                  <Label htmlFor="leyendaPie">Leyenda al pie del comprobante</Label>
                  <Input id="leyendaPie" {...form.register("leyendaPie")} />
                </div>

                <div className="flex items-center justify-between rounded-md border border-border p-3">
                  <div>
                    <p className="text-sm font-medium">Tema oscuro</p>
                    <p className="text-sm text-muted-foreground">
                      Cambia la apariencia de la aplicación.
                    </p>
                  </div>
                  <Switch
                    checked={tema === "oscuro"}
                    onCheckedChange={(v) => form.setValue("tema", v ? "oscuro" : "claro")}
                  />
                </div>

                <div className="flex items-center justify-between rounded-md border border-border p-3">
                  <div>
                    <p className="text-sm font-medium">Permitir stock negativo</p>
                    <p className="text-sm text-muted-foreground">
                      Si está desactivado, las ventas no dejarán el stock en negativo.
                    </p>
                  </div>
                  <Switch
                    checked={permiteStockNegativo}
                    onCheckedChange={(v) => form.setValue("permiteStockNegativo", v)}
                  />
                </div>

                <div className="flex items-center justify-between rounded-md border border-border p-3">
                  <div>
                    <p className="text-sm font-medium">Arqueo de caja diario</p>
                    <p className="text-sm text-muted-foreground">
                      Activa la apertura y cierre de caja.
                    </p>
                  </div>
                  <Switch
                    checked={arqueoActivo}
                    onCheckedChange={(v) => form.setValue("arqueoActivo", v)}
                  />
                </div>

                <Button type="submit" disabled={form.formState.isSubmitting}>
                  Guardar cambios
                </Button>
              </form>
            </CardContent>
          </Card>

          <SeccionPin requierePinActual={config.requierePin} />
        </TabsContent>

        <TabsContent value="respaldos">
          <RespaldosPanel />
        </TabsContent>

        <TabsContent value="actualizaciones">
          <ActualizacionesPanel />
        </TabsContent>

        <TabsContent value="historial">
          <AuditoriaPanel />
        </TabsContent>
      </Tabs>
    </div>
  );
}

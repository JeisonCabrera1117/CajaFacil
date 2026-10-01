import { useEffect, useState } from "react";
import { getVersion } from "@tauri-apps/api/app";
import { check, type Update } from "@tauri-apps/plugin-updater";
import { relaunch } from "@tauri-apps/plugin-process";
import { toast } from "sonner";
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from "@/components/ui/card";
import { Button } from "@/components/ui/button";

type Estado = "inactivo" | "buscando" | "al_dia" | "disponible" | "instalando";

export function ActualizacionesPanel() {
  const [version, setVersion] = useState("");
  const [estado, setEstado] = useState<Estado>("inactivo");
  const [update, setUpdate] = useState<Update | null>(null);

  useEffect(() => {
    getVersion()
      .then(setVersion)
      .catch(() => {});
  }, []);

  async function buscar() {
    setEstado("buscando");
    try {
      const resultado = await check();
      if (resultado) {
        setUpdate(resultado);
        setEstado("disponible");
      } else {
        setEstado("al_dia");
      }
    } catch (err) {
      setEstado("inactivo");
      toast.error(
        "No se pudo verificar actualizaciones. Es normal si todavía no se configuró un " +
          "servidor de actualizaciones. Detalle: " +
          String(err),
      );
    }
  }

  async function instalar() {
    if (!update) return;
    setEstado("instalando");
    try {
      await update.downloadAndInstall();
      await relaunch();
    } catch (err) {
      setEstado("disponible");
      toast.error(String(err));
    }
  }

  return (
    <Card>
      <CardHeader>
        <CardTitle>Actualizaciones</CardTitle>
        <CardDescription>Versión instalada: {version || "…"}</CardDescription>
      </CardHeader>
      <CardContent className="space-y-3">
        {estado === "al_dia" && (
          <p className="text-sm text-muted-foreground">Tiene la última versión disponible.</p>
        )}
        {estado === "disponible" && update && (
          <div className="space-y-2 rounded-md border border-border p-3">
            <p className="text-sm font-medium">Nueva versión disponible: {update.version}</p>
            {update.body && <p className="text-sm text-muted-foreground">{update.body}</p>}
            <Button size="sm" onClick={instalar} disabled={estado !== "disponible"}>
              Instalar y reiniciar
            </Button>
          </div>
        )}
        <Button
          variant="outline"
          onClick={buscar}
          disabled={estado === "buscando" || estado === "instalando"}
        >
          {estado === "buscando"
            ? "Buscando…"
            : estado === "instalando"
              ? "Instalando…"
              : "Buscar actualizaciones"}
        </Button>
      </CardContent>
    </Card>
  );
}

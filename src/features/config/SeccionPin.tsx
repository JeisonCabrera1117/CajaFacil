import { useState } from "react";
import { toast } from "sonner";
import { api } from "@/lib/tauri";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";

export function SeccionPin({ requierePinActual }: { requierePinActual: boolean }) {
  const [nuevoPin, setNuevoPin] = useState("");
  const [confirmarPin, setConfirmarPin] = useState("");
  const [pinActual, setPinActual] = useState("");
  const [enviando, setEnviando] = useState(false);

  async function activarOCambiar() {
    if (nuevoPin !== confirmarPin) {
      toast.error("El PIN y su confirmación no coinciden.");
      return;
    }
    setEnviando(true);
    try {
      await api.authSetPin(nuevoPin);
      toast.success(requierePinActual ? "PIN actualizado." : "PIN activado.");
      setNuevoPin("");
      setConfirmarPin("");
      window.location.reload();
    } catch (err) {
      toast.error(String(err));
    } finally {
      setEnviando(false);
    }
  }

  async function desactivar() {
    setEnviando(true);
    try {
      await api.authDisablePin(pinActual);
      toast.success("PIN desactivado.");
      setPinActual("");
      window.location.reload();
    } catch (err) {
      toast.error(String(err));
    } finally {
      setEnviando(false);
    }
  }

  return (
    <Card>
      <CardHeader>
        <CardTitle>Acceso con PIN</CardTitle>
      </CardHeader>
      <CardContent className="space-y-4">
        <p className="text-sm text-muted-foreground">
          {requierePinActual
            ? "El PIN está activo. Puede cambiarlo definiendo uno nuevo."
            : "No se solicita PIN al abrir la aplicación."}
        </p>

        <div className="grid grid-cols-2 gap-4">
          <div className="space-y-1.5">
            <Label htmlFor="nuevoPin">{requierePinActual ? "Nuevo PIN" : "PIN"}</Label>
            <Input
              id="nuevoPin"
              type="password"
              inputMode="numeric"
              maxLength={8}
              value={nuevoPin}
              onChange={(e) => setNuevoPin(e.currentTarget.value)}
            />
          </div>
          <div className="space-y-1.5">
            <Label htmlFor="confirmarPin">Confirmar PIN</Label>
            <Input
              id="confirmarPin"
              type="password"
              inputMode="numeric"
              maxLength={8}
              value={confirmarPin}
              onChange={(e) => setConfirmarPin(e.currentTarget.value)}
            />
          </div>
        </div>
        <Button onClick={activarOCambiar} disabled={enviando || nuevoPin.length < 4}>
          {requierePinActual ? "Cambiar PIN" : "Activar PIN"}
        </Button>

        {requierePinActual && (
          <div className="space-y-3 border-t border-border pt-4">
            <div className="max-w-xs space-y-1.5">
              <Label htmlFor="pinActual">PIN actual (para desactivar)</Label>
              <Input
                id="pinActual"
                type="password"
                inputMode="numeric"
                maxLength={8}
                value={pinActual}
                onChange={(e) => setPinActual(e.currentTarget.value)}
              />
            </div>
            <Button
              variant="destructive"
              onClick={desactivar}
              disabled={enviando || pinActual.length < 4}
            >
              Desactivar PIN
            </Button>
          </div>
        )}
      </CardContent>
    </Card>
  );
}

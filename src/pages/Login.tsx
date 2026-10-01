import { useState, type FormEvent } from "react";
import { useAuth } from "@/features/auth/AuthContext";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";

export function Login() {
  const { desbloquear } = useAuth();
  const [pin, setPin] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [verificando, setVerificando] = useState(false);

  async function onSubmit(e: FormEvent) {
    e.preventDefault();
    setError(null);
    setVerificando(true);
    try {
      const ok = await desbloquear(pin);
      if (!ok) setError("PIN incorrecto.");
    } catch (err) {
      setError(String(err));
    } finally {
      setVerificando(false);
      setPin("");
    }
  }

  return (
    <div className="flex min-h-screen items-center justify-center bg-muted/30 p-4">
      <Card className="w-full max-w-sm">
        <CardHeader>
          <CardTitle className="text-center">CajaFácil</CardTitle>
        </CardHeader>
        <CardContent>
          <form onSubmit={onSubmit} className="space-y-4">
            <Input
              type="password"
              inputMode="numeric"
              autoFocus
              placeholder="PIN"
              value={pin}
              onChange={(e) => setPin(e.currentTarget.value)}
              maxLength={8}
            />
            {error && <p className="text-sm text-destructive">{error}</p>}
            <Button type="submit" className="w-full" disabled={verificando || pin.length < 4}>
              Ingresar
            </Button>
          </form>
        </CardContent>
      </Card>
    </div>
  );
}

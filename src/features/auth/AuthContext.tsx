import { createContext, useContext, useEffect, useState, type ReactNode } from "react";
import { api } from "@/lib/tauri";

interface AuthContextValue {
  cargando: boolean;
  requierePin: boolean;
  desbloqueada: boolean;
  desbloquear: (pin: string) => Promise<boolean>;
}

const AuthContext = createContext<AuthContextValue | null>(null);

export function AuthProvider({ children }: { children: ReactNode }) {
  const [cargando, setCargando] = useState(true);
  const [requierePin, setRequierePin] = useState(false);
  const [desbloqueada, setDesbloqueada] = useState(false);

  useEffect(() => {
    api
      .authCheckRequired()
      .then((requerido) => {
        setRequierePin(requerido);
        setDesbloqueada(!requerido);
      })
      .finally(() => setCargando(false));
  }, []);

  async function desbloquear(pin: string) {
    const ok = await api.authVerifyPin(pin);
    if (ok) setDesbloqueada(true);
    return ok;
  }

  return (
    <AuthContext.Provider value={{ cargando, requierePin, desbloqueada, desbloquear }}>
      {children}
    </AuthContext.Provider>
  );
}

export function useAuth(): AuthContextValue {
  const ctx = useContext(AuthContext);
  if (!ctx) throw new Error("useAuth debe usarse dentro de <AuthProvider>");
  return ctx;
}

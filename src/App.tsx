import { BrowserRouter, Routes, Route } from "react-router-dom";
import { QueryClientProvider } from "@tanstack/react-query";
import { ThemeProvider } from "next-themes";
import { Toaster } from "@/components/ui/sonner";
import { queryClient } from "@/lib/queryClient";
import { AuthProvider, useAuth } from "@/features/auth/AuthContext";
import { Login } from "@/pages/Login";
import { AppShell } from "@/layout/AppShell";
import { Dashboard } from "@/pages/Dashboard";
import { Inventario } from "@/pages/Inventario";
import { Ventas } from "@/pages/Ventas";
import { Clientes } from "@/pages/Clientes";
import { Proveedores } from "@/pages/Proveedores";
import { Reportes } from "@/pages/Reportes";
import { CargaMasiva } from "@/pages/CargaMasiva";
import { Configuracion } from "@/pages/Configuracion";

function Contenido() {
  const { cargando, requierePin, desbloqueada } = useAuth();

  if (cargando) {
    return (
      <div className="flex min-h-screen items-center justify-center text-muted-foreground">
        Cargando…
      </div>
    );
  }

  if (requierePin && !desbloqueada) {
    return <Login />;
  }

  return (
    <Routes>
      <Route element={<AppShell />}>
        <Route index element={<Dashboard />} />
        <Route path="inventario" element={<Inventario />} />
        <Route path="ventas" element={<Ventas />} />
        <Route path="clientes" element={<Clientes />} />
        <Route path="proveedores" element={<Proveedores />} />
        <Route path="reportes" element={<Reportes />} />
        <Route path="carga-masiva" element={<CargaMasiva />} />
        <Route path="configuracion" element={<Configuracion />} />
      </Route>
    </Routes>
  );
}

function App() {
  return (
    <QueryClientProvider client={queryClient}>
      <ThemeProvider attribute="class" defaultTheme="light" enableSystem={false}>
        <AuthProvider>
          <BrowserRouter>
            <Contenido />
          </BrowserRouter>
        </AuthProvider>
        <Toaster />
      </ThemeProvider>
    </QueryClientProvider>
  );
}

export default App;

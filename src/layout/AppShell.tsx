import { NavLink, Outlet } from "react-router-dom";
import { useQuery } from "@tanstack/react-query";
import { useTheme } from "next-themes";
import { useEffect } from "react";
import {
  LayoutDashboard,
  Package,
  ShoppingCart,
  Users,
  Truck,
  BarChart3,
  Upload,
  Settings,
} from "lucide-react";
import { cn } from "@/lib/utils";
import { api } from "@/lib/tauri";

const ENLACES = [
  { to: "/", label: "Dashboard", icon: LayoutDashboard, end: true },
  { to: "/inventario", label: "Inventario", icon: Package, end: false },
  { to: "/ventas", label: "Ventas / POS", icon: ShoppingCart, end: false },
  { to: "/clientes", label: "Clientes", icon: Users, end: false },
  { to: "/proveedores", label: "Proveedores", icon: Truck, end: false },
  { to: "/reportes", label: "Reportes", icon: BarChart3, end: false },
  { to: "/carga-masiva", label: "Carga masiva", icon: Upload, end: false },
  { to: "/configuracion", label: "Configuración", icon: Settings, end: false },
];

function useSincronizarTema() {
  const { setTheme } = useTheme();
  const { data } = useQuery({ queryKey: ["config"], queryFn: api.configGet });

  const tema = data?.tema;
  useEffect(() => {
    if (tema) setTheme(tema === "oscuro" ? "dark" : "light");
  }, [tema, setTheme]);
}

export function AppShell() {
  useSincronizarTema();

  return (
    <div className="flex h-screen w-screen overflow-hidden bg-background text-foreground">
      <aside className="flex w-56 shrink-0 flex-col border-r border-border bg-card">
        <div className="flex items-center gap-2 px-4 py-4 font-heading text-base font-semibold">
          <span className="size-5 shrink-0 rounded-md bg-gradient-to-br from-primary to-[#a25ddc]" />
          CajaFácil
        </div>
        <nav className="flex flex-1 flex-col gap-1 px-2">
          {ENLACES.map(({ to, label, icon: Icon, end }) => (
            <NavLink
              key={to}
              to={to}
              end={end}
              className={({ isActive }) =>
                cn(
                  "flex items-center gap-2 rounded-md px-3 py-2 text-sm font-medium transition-colors",
                  isActive
                    ? "bg-primary/10 text-primary"
                    : "text-muted-foreground hover:bg-muted hover:text-foreground",
                )
              }
            >
              <Icon className="size-4" />
              {label}
            </NavLink>
          ))}
        </nav>
      </aside>
      <main className="flex-1 overflow-y-auto p-6">
        <Outlet />
      </main>
    </div>
  );
}

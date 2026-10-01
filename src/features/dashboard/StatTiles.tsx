import {
  TrendingUp,
  CalendarDays,
  CalendarRange,
  Receipt,
  Hash,
  PiggyBank,
  Boxes,
  TriangleAlert,
  PackagePlus,
  type LucideIcon,
} from "lucide-react";
import type { DashboardIndicadores } from "@/lib/api-dashboard";
import { formatearMoneda } from "@/lib/format";
import { Card, CardContent } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { cn } from "@/lib/utils";

interface Props {
  indicadores: DashboardIndicadores;
}

const COLOR_CHIP: Record<"primary" | "success" | "warning", string> = {
  primary: "bg-primary/10 text-primary",
  success: "bg-success/10 text-success",
  warning: "bg-warning/10 text-warning",
};

function Chip({ icon: Icon, color }: { icon: LucideIcon; color: keyof typeof COLOR_CHIP }) {
  return (
    <span
      className={cn(
        "flex size-7 shrink-0 items-center justify-center rounded-lg",
        COLOR_CHIP[color],
      )}
    >
      <Icon className="size-3.5" />
    </span>
  );
}

export function StatTiles({ indicadores: ind }: Props) {
  const tiles: {
    etiqueta: string;
    valor: string;
    icon: LucideIcon;
    color: "primary" | "success";
  }[] = [
    {
      etiqueta: "Ventas de hoy",
      valor: formatearMoneda(ind.ventasHoy),
      icon: TrendingUp,
      color: "primary",
    },
    {
      etiqueta: "Ventas de la semana",
      valor: formatearMoneda(ind.ventasSemana),
      icon: CalendarDays,
      color: "primary",
    },
    {
      etiqueta: "Ventas del mes",
      valor: formatearMoneda(ind.ventasMes),
      icon: CalendarRange,
      color: "primary",
    },
    {
      etiqueta: "Ticket promedio (mes)",
      valor: formatearMoneda(ind.ticketPromedioMes),
      icon: Receipt,
      color: "primary",
    },
    {
      etiqueta: "Número de ventas (mes)",
      valor: String(ind.numeroVentasMes),
      icon: Hash,
      color: "primary",
    },
    {
      etiqueta: "Utilidad bruta (mes)",
      valor: formatearMoneda(ind.utilidadBrutaMes),
      icon: PiggyBank,
      color: "success",
    },
    {
      etiqueta: "Valor del inventario",
      valor: formatearMoneda(ind.valorInventario),
      icon: Boxes,
      color: "primary",
    },
  ];

  return (
    <div className="grid grid-cols-2 gap-3 sm:grid-cols-3 lg:grid-cols-4">
      {tiles.map((t) => (
        <Card key={t.etiqueta}>
          <CardContent className="flex items-start justify-between gap-2 p-4">
            <div className="min-w-0">
              <p className="truncate text-sm text-muted-foreground">{t.etiqueta}</p>
              <p className="mt-1 text-xl font-semibold">{t.valor}</p>
            </div>
            <Chip icon={t.icon} color={t.color} />
          </CardContent>
        </Card>
      ))}
      <Card>
        <CardContent className="flex items-start justify-between gap-2 p-4">
          <div className="min-w-0">
            <p className="text-sm text-muted-foreground">Productos con stock bajo</p>
            <div className="mt-1 flex items-center gap-2">
              <p className="text-xl font-semibold">{ind.productosStockBajo}</p>
              {ind.productosStockBajo > 0 && <Badge variant="warning">Revisar</Badge>}
            </div>
          </div>
          <Chip icon={TriangleAlert} color="warning" />
        </CardContent>
      </Card>
      <Card>
        <CardContent className="flex items-start justify-between gap-2 p-4">
          <div className="min-w-0">
            <p className="text-sm text-muted-foreground">Productos con sobre-stock</p>
            <div className="mt-1 flex items-center gap-2">
              <p className="text-xl font-semibold">{ind.productosSobreStock}</p>
              {ind.productosSobreStock > 0 && <Badge variant="warning">Revisar</Badge>}
            </div>
          </div>
          <Chip icon={PackagePlus} color="warning" />
        </CardContent>
      </Card>
    </div>
  );
}

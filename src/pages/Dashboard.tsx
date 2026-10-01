import { useState } from "react";
import { useQuery } from "@tanstack/react-query";
import { apiDashboard } from "@/lib/api-dashboard";
import { primerDiaMesISO, hoyISO } from "@/lib/format";
import { StatTiles } from "@/features/dashboard/StatTiles";
import { GraficaVentas } from "@/features/dashboard/GraficaVentas";
import { GraficaProductos } from "@/features/dashboard/GraficaProductos";
import { GraficaDonas } from "@/features/dashboard/GraficaDonas";
import { Comparativo } from "@/features/dashboard/Comparativo";
import { AlertasStock } from "@/features/dashboard/AlertasStock";

export function Dashboard() {
  const [desde, setDesde] = useState(primerDiaMesISO());
  const [hasta, setHasta] = useState(hoyISO());

  const { data: indicadores } = useQuery({
    queryKey: ["dashboard-indicadores"],
    queryFn: apiDashboard.indicadores,
  });
  const { data: graficas } = useQuery({
    queryKey: ["dashboard-graficas", desde, hasta],
    queryFn: () => apiDashboard.graficas(desde, hasta),
  });

  return (
    <div className="space-y-6">
      <h1 className="text-2xl font-semibold">Dashboard</h1>

      {indicadores && <StatTiles indicadores={indicadores} />}

      <AlertasStock />

      {graficas && (
        <>
          <GraficaVentas
            serie={graficas.serieVentas}
            desde={desde}
            hasta={hasta}
            onDesdeChange={setDesde}
            onHastaChange={setHasta}
          />
          <Comparativo comparativo={graficas.comparativo} />
          <GraficaProductos
            masVendidos={graficas.productosMasVendidos}
            menosVendidos={graficas.productosMenosVendidos}
          />
          <GraficaDonas
            porCategoria={graficas.ventasPorCategoria}
            porMetodoPago={graficas.ventasPorMetodoPago}
          />
        </>
      )}
    </div>
  );
}

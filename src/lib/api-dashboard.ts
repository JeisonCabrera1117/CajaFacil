import { invoke } from "@tauri-apps/api/core";

export interface DashboardIndicadores {
  ventasHoy: number;
  ventasSemana: number;
  ventasMes: number;
  numeroVentasMes: number;
  ticketPromedioMes: number;
  utilidadBrutaMes: number;
  productosStockBajo: number;
  productosSobreStock: number;
  valorInventario: number;
}

export interface PuntoSerie {
  etiqueta: string;
  total: number;
}

export interface ProductoCantidad {
  nombre: string;
  cantidad: number;
}

export interface Comparativo {
  actual: number;
  anterior: number;
}

export interface DashboardGraficas {
  serieVentas: PuntoSerie[];
  productosMasVendidos: ProductoCantidad[];
  productosMenosVendidos: ProductoCantidad[];
  ventasPorCategoria: PuntoSerie[];
  ventasPorMetodoPago: PuntoSerie[];
  comparativo: Comparativo;
}

export const apiDashboard = {
  indicadores: () => invoke<DashboardIndicadores>("dashboard_indicadores"),
  graficas: (desde?: string, hasta?: string) =>
    invoke<DashboardGraficas>("dashboard_graficas", { desde: desde ?? null, hasta: hasta ?? null }),
};

import { invoke } from "@tauri-apps/api/core";
import type { PlantillaGenerada } from "@/lib/api-importacion";

export interface FiltroReporteVentas {
  desde?: string;
  hasta?: string;
  productoId?: number | null;
  categoriaId?: number | null;
  clienteId?: number | null;
}

export interface FilaReporteVenta {
  fecha: string;
  numeroComprobante: string;
  cliente: string;
  producto: string;
  categoria: string | null;
  cantidad: number;
  precioUnitario: number;
  subtotal: number;
  estado: string;
}

export interface FilaUtilidad {
  producto: string;
  cantidadVendida: number;
  ingresos: number;
  costo: number;
  utilidad: number;
}

export interface FilaInventarioValorizado {
  sku: string;
  nombre: string;
  categoria: string | null;
  stockActual: number;
  costoUnitario: number;
  valorTotal: number;
}

export interface FilaRotacion {
  sku: string;
  nombre: string;
  cantidadVendida: number;
  stockActual: number;
  sinMovimiento: boolean;
}

export interface FilaComprasProveedor {
  proveedor: string;
  numeroCompras: number;
  totalComprado: number;
}

export type FormatoExportacion = "csv" | "xlsx" | "pdf";

export const apiReportes = {
  ventas: (filtro: FiltroReporteVentas) => invoke<FilaReporteVenta[]>("reporte_ventas", { filtro }),
  utilidad: (desde: string, hasta: string) =>
    invoke<FilaUtilidad[]>("reporte_utilidad", { desde, hasta }),
  inventarioValorizado: () => invoke<FilaInventarioValorizado[]>("reporte_inventario_valorizado"),
  rotacion: (desde: string, hasta: string) =>
    invoke<FilaRotacion[]>("reporte_rotacion", { desde, hasta }),
  comprasPorProveedor: (desde: string, hasta: string) =>
    invoke<FilaComprasProveedor[]>("reporte_compras_proveedor", { desde, hasta }),
  exportar: (titulo: string, columnas: string[], filas: string[][], formato: FormatoExportacion) =>
    invoke<PlantillaGenerada>("reporte_exportar", {
      solicitud: { titulo, columnas, filas, formato },
    }),
};

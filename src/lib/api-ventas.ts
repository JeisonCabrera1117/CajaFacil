import { invoke } from "@tauri-apps/api/core";
import type { Producto } from "@/lib/api-inventario";

export interface Cliente {
  id: number;
  documento: string | null;
  nombre: string;
  telefono: string | null;
  correo: string | null;
  direccion: string | null;
}

export type ClienteNuevo = Omit<Cliente, "id">;

export type MetodoPago = "efectivo" | "tarjeta" | "transferencia" | "mixto";

export interface VentaItemNuevo {
  productoId: number;
  cantidad: number;
  precioUnitario: number;
  descuentoPct: number;
  descuentoValor: number;
  impuestoPct: number;
}

export interface VentaNueva {
  clienteId: number | null;
  metodoPago: MetodoPago;
  valorRecibido: number | null;
  descuentoGlobal: number;
  items: VentaItemNuevo[];
}

export interface VentaDetalleItem {
  id: number;
  productoId: number;
  productoNombre: string;
  cantidad: number;
  precioUnitario: number;
  descuentoPct: number;
  descuentoValor: number;
  impuestoPct: number;
  subtotal: number;
  cantidadDevuelta: number;
}

export interface Venta {
  id: number;
  numeroComprobante: string;
  fecha: string;
  clienteId: number | null;
  clienteNombre: string | null;
  subtotal: number;
  descuentoTotal: number;
  impuestos: number;
  total: number;
  metodoPago: MetodoPago;
  valorRecibido: number | null;
  cambio: number | null;
  estado: "completada" | "anulada";
  motivoAnulacion: string | null;
  cajaSesionId: number | null;
}

export type VentaConDetalle = { venta: Venta; items: VentaDetalleItem[] };

export interface VentaFiltro {
  desde?: string;
  hasta?: string;
  clienteId?: number | null;
  estado?: string;
  metodoPago?: string;
}

export interface DevolucionItemNuevo {
  ventaDetalleId: number;
  cantidad: number;
}

export interface DevolucionNueva {
  ventaId: number;
  motivo: string;
  items: DevolucionItemNuevo[];
}

export interface DevolucionDetalleItem {
  id: number;
  ventaDetalleId: number;
  productoNombre: string;
  cantidad: number;
  valor: number;
}

export interface Devolucion {
  id: number;
  ventaId: number;
  fecha: string;
  motivo: string;
  totalDevuelto: number;
  items: DevolucionDetalleItem[];
}

export interface CajaSesion {
  id: number;
  fechaApertura: string;
  fechaCierre: string | null;
  montoApertura: number;
  montoCierreSistema: number | null;
  montoCierreContado: number | null;
  diferencia: number | null;
  observaciones: string | null;
}

export const apiVentas = {
  clienteList: () => invoke<Cliente[]>("cliente_list"),
  clienteGet: (id: number) => invoke<Cliente>("cliente_get", { id }),
  clienteCreate: (datos: ClienteNuevo) => invoke<Cliente>("cliente_create", { datos }),
  clienteUpdate: (id: number, datos: ClienteNuevo) =>
    invoke<Cliente>("cliente_update", { id, datos }),

  cajaEstadoActual: () => invoke<CajaSesion | null>("caja_estado_actual"),
  cajaAbrir: (montoApertura: number) => invoke<CajaSesion>("caja_abrir", { montoApertura }),
  cajaCerrar: (montoCierreContado: number, observaciones: string | null) =>
    invoke<CajaSesion>("caja_cerrar", { montoCierreContado, observaciones }),

  ventaBuscarProducto: (termino: string) =>
    invoke<Producto[]>("venta_buscar_producto", { termino }),
  ventaCrear: (datos: VentaNueva) => invoke<VentaConDetalle>("venta_crear", { datos }),
  ventaAnular: (id: number, motivo: string) =>
    invoke<VentaConDetalle>("venta_anular", { id, motivo }),
  ventaList: (filtro: VentaFiltro) => invoke<Venta[]>("venta_list", { filtro }),
  ventaGet: (id: number) => invoke<VentaConDetalle>("venta_get", { id }),

  devolucionCrear: (datos: DevolucionNueva) => invoke<Devolucion>("devolucion_crear", { datos }),
};

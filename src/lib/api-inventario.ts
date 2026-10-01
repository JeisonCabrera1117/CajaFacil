import { invoke } from "@tauri-apps/api/core";

export interface Categoria {
  id: number;
  nombre: string;
  categoriaPadreId: number | null;
}

export interface CategoriaNueva {
  nombre: string;
  categoriaPadreId: number | null;
}

export interface Proveedor {
  id: number;
  nit: string | null;
  razonSocial: string;
  contacto: string | null;
  telefono: string | null;
  correo: string | null;
  direccion: string | null;
  condicionesPago: string | null;
  activo: boolean;
}

export type ProveedorNuevo = Omit<Proveedor, "id">;

export interface Producto {
  id: number;
  sku: string;
  codigoBarras: string | null;
  nombre: string;
  descripcion: string | null;
  categoriaId: number | null;
  unidadMedida: string;
  precioCosto: number;
  precioVenta: number;
  impuestoPct: number;
  stockActual: number;
  stockMinimo: number;
  stockMaximo: number | null;
  ubicacion: string | null;
  proveedorPrincipalId: number | null;
  estado: "activo" | "inactivo";
  imagenPath: string | null;
}

export type ProductoNuevo = Omit<Producto, "id" | "stockActual">;

export interface ProductoFiltro {
  busqueda?: string;
  categoriaId?: number | null;
  soloStockBajo?: boolean;
  soloSobreStock?: boolean;
  pagina?: number;
  porPagina?: number;
}

export interface ProductoPagina {
  items: Producto[];
  total: number;
}

export type TipoMovimiento = "entrada" | "salida" | "ajuste" | "devolucion";

export interface MovimientoStock {
  id: number;
  productoId: number;
  tipo: TipoMovimiento;
  cantidad: number;
  costoUnitario: number | null;
  motivo: string | null;
  referenciaTipo: string | null;
  referenciaId: number | null;
  fecha: string;
  saldoResultante: number;
}

export interface AjustarStockEntrada {
  productoId: number;
  tipo: TipoMovimiento;
  cantidad: number;
  costoUnitario?: number | null;
  motivo?: string | null;
}

export interface CompraItemNuevo {
  productoId: number;
  cantidad: number;
  costoUnitario: number;
}

export interface CompraNueva {
  proveedorId: number | null;
  numero: string;
  observaciones: string | null;
  items: CompraItemNuevo[];
}

export interface Compra {
  id: number;
  proveedorId: number | null;
  proveedorNombre: string | null;
  numero: string;
  fecha: string;
  subtotal: number;
  impuestos: number;
  total: number;
  estado: "registrada" | "anulada";
  observaciones: string | null;
}

export interface CompraDetalleItem {
  id: number;
  productoId: number;
  productoNombre: string;
  cantidad: number;
  costoUnitario: number;
  subtotal: number;
}

export type CompraConDetalle = Compra & { items: CompraDetalleItem[] };

export interface InventarioFisico {
  id: number;
  fecha: string;
  estado: "abierto" | "cerrado";
  observaciones: string | null;
}

export interface InventarioFisicoDetalle {
  id: number;
  productoId: number;
  productoNombre: string;
  sku: string;
  stockSistema: number;
  stockContado: number;
  diferencia: number;
}

export interface InventarioFisicoConDetalle {
  cabecera: InventarioFisico;
  detalle: InventarioFisicoDetalle[];
}

export interface RegistrarConteo {
  inventarioFisicoId: number;
  productoId: number;
  stockContado: number;
}

export const apiInventario = {
  categoriaList: () => invoke<Categoria[]>("categoria_list"),
  categoriaCreate: (datos: CategoriaNueva) => invoke<Categoria>("categoria_create", { datos }),
  categoriaUpdate: (id: number, datos: CategoriaNueva) =>
    invoke<Categoria>("categoria_update", { id, datos }),
  categoriaDelete: (id: number) => invoke<void>("categoria_delete", { id }),

  proveedorList: () => invoke<Proveedor[]>("proveedor_list"),
  proveedorGet: (id: number) => invoke<Proveedor>("proveedor_get", { id }),
  proveedorCreate: (datos: ProveedorNuevo) => invoke<Proveedor>("proveedor_create", { datos }),
  proveedorUpdate: (id: number, datos: ProveedorNuevo) =>
    invoke<Proveedor>("proveedor_update", { id, datos }),
  proveedorDelete: (id: number) => invoke<void>("proveedor_delete", { id }),
  proveedorHistorialCompras: (id: number) =>
    invoke<Compra[]>("proveedor_historial_compras", { id }),
  proveedorProductosList: (id: number) => invoke<number[]>("proveedor_productos_list", { id }),
  proveedorProductosAsignar: (id: number, productoIds: number[]) =>
    invoke<void>("proveedor_productos_asignar", { id, productoIds }),

  productoList: (filtro: ProductoFiltro) => invoke<ProductoPagina>("producto_list", { filtro }),
  productoGet: (id: number) => invoke<Producto>("producto_get", { id }),
  productoCreate: (datos: ProductoNuevo) => invoke<Producto>("producto_create", { datos }),
  productoUpdate: (id: number, datos: ProductoNuevo) =>
    invoke<Producto>("producto_update", { id, datos }),
  productoDelete: (id: number) => invoke<void>("producto_delete", { id }),
  productoAjustarStock: (datos: AjustarStockEntrada) =>
    invoke<MovimientoStock>("producto_ajustar_stock", { datos }),
  kardexGet: (productoId: number) => invoke<MovimientoStock[]>("kardex_get", { productoId }),

  compraList: (proveedorId: number | null) => invoke<Compra[]>("compra_list", { proveedorId }),
  compraGet: (id: number) => invoke<CompraConDetalle>("compra_get", { id }),
  compraCreate: (datos: CompraNueva) => invoke<CompraConDetalle>("compra_create", { datos }),
  compraAnular: (id: number, motivo: string) =>
    invoke<CompraConDetalle>("compra_anular", { id, motivo }),

  inventarioFisicoIniciar: (observaciones: string | null) =>
    invoke<InventarioFisico>("inventario_fisico_iniciar", { observaciones }),
  inventarioFisicoList: () => invoke<InventarioFisico[]>("inventario_fisico_list"),
  inventarioFisicoGet: (id: number) =>
    invoke<InventarioFisicoConDetalle>("inventario_fisico_get", { id }),
  inventarioFisicoRegistrarConteo: (datos: RegistrarConteo) =>
    invoke<InventarioFisicoConDetalle>("inventario_fisico_registrar_conteo", { datos }),
  inventarioFisicoCerrar: (id: number) =>
    invoke<InventarioFisicoConDetalle>("inventario_fisico_cerrar", { id }),
};

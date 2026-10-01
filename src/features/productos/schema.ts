import { z } from "zod";

const numeroTexto = (mensaje: string) =>
  z
    .string()
    .min(1, mensaje)
    .refine((v) => !Number.isNaN(Number(v)), "Debe ser un número válido");

export const productoSchema = z.object({
  sku: z.string().min(1, "El SKU es obligatorio"),
  codigoBarras: z.string(),
  nombre: z.string().min(1, "El nombre es obligatorio"),
  descripcion: z.string(),
  categoriaId: z.string(),
  unidadMedida: z.string().min(1, "La unidad de medida es obligatoria"),
  precioCosto: numeroTexto("El precio de costo es obligatorio"),
  precioVenta: numeroTexto("El precio de venta es obligatorio"),
  impuestoPct: numeroTexto("El impuesto es obligatorio"),
  stockMinimo: numeroTexto("El stock mínimo es obligatorio"),
  stockMaximo: z.string(),
  ubicacion: z.string(),
  proveedorPrincipalId: z.string(),
  estado: z.enum(["activo", "inactivo"]),
  imagenPath: z.string(),
});

export type ProductoFormValues = z.infer<typeof productoSchema>;

export const valoresPorDefecto: ProductoFormValues = {
  sku: "",
  codigoBarras: "",
  nombre: "",
  descripcion: "",
  categoriaId: "none",
  unidadMedida: "unidad",
  precioCosto: "0",
  precioVenta: "0",
  impuestoPct: "0",
  stockMinimo: "0",
  stockMaximo: "",
  ubicacion: "",
  proveedorPrincipalId: "none",
  estado: "activo",
  imagenPath: "",
};

import { z } from "zod";

export const proveedorSchema = z.object({
  nit: z.string(),
  razonSocial: z.string().min(1, "La razón social es obligatoria"),
  contacto: z.string(),
  telefono: z.string(),
  correo: z.string(),
  direccion: z.string(),
  condicionesPago: z.string(),
  activo: z.boolean(),
});

export type ProveedorFormValues = z.infer<typeof proveedorSchema>;

export const valoresPorDefecto: ProveedorFormValues = {
  nit: "",
  razonSocial: "",
  contacto: "",
  telefono: "",
  correo: "",
  direccion: "",
  condicionesPago: "",
  activo: true,
};

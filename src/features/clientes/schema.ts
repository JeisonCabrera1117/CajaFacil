import { z } from "zod";

export const clienteSchema = z.object({
  documento: z.string(),
  nombre: z.string().min(1, "El nombre es obligatorio"),
  telefono: z.string(),
  correo: z.string(),
  direccion: z.string(),
});

export type ClienteFormValues = z.infer<typeof clienteSchema>;

export const valoresPorDefecto: ClienteFormValues = {
  documento: "",
  nombre: "",
  telefono: "",
  correo: "",
  direccion: "",
};

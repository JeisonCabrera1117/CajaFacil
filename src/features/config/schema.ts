import { z } from "zod";

export const empresaConfigSchema = z.object({
  nombre: z.string().min(1, "El nombre es obligatorio"),
  nit: z.string(),
  direccion: z.string(),
  telefono: z.string(),
  logoPath: z.string(),
  moneda: z.string().min(1, "La moneda es obligatoria"),
  formatoFecha: z.enum(["DD/MM/YYYY", "MM/DD/YYYY", "YYYY-MM-DD"]),
  prefijoComprobante: z.string().min(1, "El prefijo es obligatorio"),
  leyendaPie: z.string(),
  tema: z.enum(["claro", "oscuro"]),
  permiteStockNegativo: z.boolean(),
  arqueoActivo: z.boolean(),
});

export type EmpresaConfigFormValues = z.infer<typeof empresaConfigSchema>;

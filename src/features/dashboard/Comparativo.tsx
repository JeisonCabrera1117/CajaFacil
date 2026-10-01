import { TrendingDown, TrendingUp, Minus } from "lucide-react";
import type { Comparativo as ComparativoTipo } from "@/lib/api-dashboard";
import { formatearMoneda } from "@/lib/format";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";

export function Comparativo({ comparativo }: { comparativo: ComparativoTipo }) {
  const { actual, anterior } = comparativo;
  const variacionPct = anterior > 0 ? ((actual - anterior) / anterior) * 100 : actual > 0 ? 100 : 0;
  const subio = variacionPct > 0.5;
  const bajo = variacionPct < -0.5;

  return (
    <Card>
      <CardHeader>
        <CardTitle className="text-base">Periodo actual vs. periodo anterior</CardTitle>
      </CardHeader>
      <CardContent className="flex flex-wrap items-center gap-6">
        <div>
          <p className="text-xs text-muted-foreground">Periodo actual</p>
          <p className="text-2xl font-semibold">{formatearMoneda(actual)}</p>
        </div>
        <div>
          <p className="text-xs text-muted-foreground">Periodo anterior (misma duración)</p>
          <p className="text-2xl font-semibold text-muted-foreground">
            {formatearMoneda(anterior)}
          </p>
        </div>
        <Badge
          variant={subio ? "success" : bajo ? "destructive" : "secondary"}
          className="gap-1 text-sm"
        >
          {subio && <TrendingUp className="size-4" />}
          {bajo && <TrendingDown className="size-4" />}
          {!subio && !bajo && <Minus className="size-4" />}
          {variacionPct >= 0 ? "+" : ""}
          {variacionPct.toFixed(1)}%
        </Badge>
      </CardContent>
    </Card>
  );
}

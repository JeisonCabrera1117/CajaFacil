export function Placeholder({ titulo, fase }: { titulo: string; fase: string }) {
  return (
    <div>
      <h1 className="text-2xl font-semibold">{titulo}</h1>
      <p className="mt-2 text-muted-foreground">Este módulo se construye en la {fase}.</p>
    </div>
  );
}

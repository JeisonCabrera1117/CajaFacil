import { describe, expect, it } from "vitest";
import { render, screen } from "@testing-library/react";
import { Placeholder } from "@/pages/Placeholder";

describe("Placeholder", () => {
  it("muestra el título y la fase indicada", () => {
    render(<Placeholder titulo="Inventario" fase="fase 2" />);
    expect(screen.getByRole("heading", { name: "Inventario" })).toBeInTheDocument();
    expect(screen.getByText(/fase 2/)).toBeInTheDocument();
  });
});

/** Rasteriza el <svg> de una gráfica de Recharts a PNG y dispara la descarga. */
export function exportarGraficaComoPng(contenedor: HTMLElement | null, nombreArchivo: string) {
  const svg = contenedor?.querySelector("svg");
  if (!svg) return;

  const clon = svg.cloneNode(true) as SVGSVGElement;
  const estiloFondo = getComputedStyle(document.body).getPropertyValue("--background") || "#ffffff";
  clon.style.backgroundColor = `oklch(${estiloFondo})`;

  const ancho = svg.clientWidth || Number(svg.getAttribute("width")) || 600;
  const alto = svg.clientHeight || Number(svg.getAttribute("height")) || 400;
  clon.setAttribute("width", String(ancho));
  clon.setAttribute("height", String(alto));

  const datosSvg = new XMLSerializer().serializeToString(clon);
  const svgUrl = `data:image/svg+xml;charset=utf-8,${encodeURIComponent(datosSvg)}`;

  const imagen = new Image();
  imagen.onload = () => {
    const escala = 2; // exporta a 2x para que no se vea pixelado
    const canvas = document.createElement("canvas");
    canvas.width = ancho * escala;
    canvas.height = alto * escala;
    const contexto = canvas.getContext("2d");
    if (!contexto) return;
    contexto.scale(escala, escala);
    contexto.fillStyle = "white";
    contexto.fillRect(0, 0, ancho, alto);
    contexto.drawImage(imagen, 0, 0, ancho, alto);

    canvas.toBlob((blob) => {
      if (!blob) return;
      const url = URL.createObjectURL(blob);
      const enlace = document.createElement("a");
      enlace.href = url;
      enlace.download = nombreArchivo.endsWith(".png") ? nombreArchivo : `${nombreArchivo}.png`;
      document.body.appendChild(enlace);
      enlace.click();
      enlace.remove();
      URL.revokeObjectURL(url);
    }, "image/png");
  };
  imagen.src = svgUrl;
}

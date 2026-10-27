# Giz na Tela — © 2026 Heraldo Antunes — Licença MIT (ver LICENSE). Gera icons/icon.ico e docs/img/icone.png
"""Ícone do Giz na Tela (decisão do professor, 26/10/2026): monitor verde com um risco de giz branco, nas cores do Giz
Livre. O mesmo desenho do botão da barra recolhida e do cabeçalho da ajuda (viewBox 64 x 64). Cada tamanho é
desenhado 8 vezes maior e reduzido, para o 16 px continuar legível. Precisa do Pillow (só para gerar; o programa não usa).
Uso: python scripts/gerar_icone.py"""
from pathlib import Path

from PIL import Image, ImageDraw

RAIZ = Path(__file__).resolve().parent.parent
VERDE, BRANCO, GIZ = (31, 107, 69, 255), (255, 255, 255, 255), (246, 242, 231, 255)


def bezier(p0, p1, p2, p3, n=24):
    return [tuple((1 - t) ** 3 * a + 3 * (1 - t) ** 2 * t * b + 3 * (1 - t) * t * t * c + t ** 3 * d
                  for a, b, c, d in zip(p0, p1, p2, p3)) for t in (i / n for i in range(n + 1))]


def linha(d, pts, k, larg, cor):
    pts = [(x * k, y * k) for x, y in pts]
    d.line(pts, fill=cor, width=round(larg * k), joint="curve")
    r = larg * k / 2
    for x, y in (pts[0], pts[-1]):   # pontas redondas
        d.ellipse((x - r, y - r, x + r, y + r), fill=cor)


def desenhar(tam):
    k = tam * 8 / 64
    img = Image.new("RGBA", (tam * 8, tam * 8), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    d.rounded_rectangle((2 * k, 2 * k, 62 * k, 62 * k), radius=14 * k, fill=VERDE)
    # nos tamanhos pequenos os traços engrossam um pouco, senão somem na redução
    g = 1.0 if tam >= 48 else 1.25 if tam >= 24 else 1.5
    d.rounded_rectangle((11 * k, 14 * k, 53 * k, 42 * k), radius=3 * k, outline=BRANCO, width=round(3.4 * g * k))
    # risco de giz: M18 33 c5-7 9-8 13-4 s7 1 10-5 (o mesmo caminho do SVG)
    giz = bezier((18, 33), (23, 26), (27, 25), (31, 29)) + bezier((31, 29), (35, 33), (38, 30), (41, 24))[1:]
    linha(d, giz, k, 3.6 * g, GIZ)
    linha(d, [(24, 51), (40, 51)], k, 3.4 * g, BRANCO)
    linha(d, [(32, 42), (32, 51)], k, 3.4 * g, BRANCO)
    return img.resize((tam, tam), Image.LANCZOS)


def main():
    tamanhos = [16, 20, 24, 32, 40, 48, 64, 128, 256]
    imgs = {t: desenhar(t) for t in tamanhos}
    ico = RAIZ / "icons" / "icon.ico"
    imgs[256].save(ico, format="ICO", sizes=[(t, t) for t in tamanhos], append_images=[imgs[t] for t in tamanhos[:-1]])
    (RAIZ / "docs" / "img").mkdir(parents=True, exist_ok=True)
    imgs[256].save(RAIZ / "docs" / "img" / "icone.png")
    imgs[128].save(RAIZ / "web" / "icone.png")
    print("ok:", ico)


if __name__ == "__main__":
    main()

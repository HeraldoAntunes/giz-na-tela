# Giz na Tela — © 2026 Heraldo Antunes — Licença MIT (ver LICENSE). Foto da capa: o anotador por cima de um PDF
"""Foto real do Giz na Tela por cima de um PDF aberto no Edge (apostila de exemplo, conteúdo próprio, ao lado deste
script; o PDF sai de apostila.html impressa pelo Edge). Grava docs/img/capa-foto.png; depois rode capa.py.
Ocupa a tela principal por cerca de 30 s: avise antes e não mexa no mouse. Uso: python scripts/fotos/foto_capa.py"""
import ctypes, math, shutil, subprocess, sys, tempfile, time
from pathlib import Path
from PIL import ImageGrab

GNT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(GNT / "tests"))
from testar_comandos import Anotador  # noqa: E402

AQUI = Path(__file__).parent
PDF = AQUI / "apostila-lei-de-ohm.pdf"
IMG = GNT / "docs" / "img"
EDGE = Path(r"C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe")
TITULO = "Apostila de F"


def janela_por_cima():
    u = ctypes.windll.user32
    achadas = []

    @ctypes.WINFUNCTYPE(ctypes.c_bool, ctypes.c_void_p, ctypes.c_void_p)
    def cada(h, _):
        n = u.GetWindowTextLengthW(h)
        b = ctypes.create_unicode_buffer(n + 1)
        u.GetWindowTextW(h, b, n + 1)
        if (TITULO in b.value or "apostila-lei-de-ohm" in b.value) and u.IsWindowVisible(h):
            achadas.append(h)
        return True

    u.EnumWindows(cada, None)
    for h in achadas:
        u.SetWindowPos(ctypes.c_void_p(h), ctypes.c_void_p(-1), 0, 0, 0, 0, 0x0001 | 0x0002 | 0x0040)
    return bool(achadas)


def circulo(cx, cy, rx, ry, voltas=1.08, n=70):
    pts = []
    for i in range(n + 1):
        t = i / n * voltas * 2 * math.pi - 0.6
        f = 1 + 0.04 * math.sin(3 * t)
        pts.append([cx + rx * f * math.cos(t), cy + ry * f * math.sin(t), round(0.35 + 0.5 * math.sin(math.pi * i / n), 2)])
    return pts


def main():
    tmp = Path(tempfile.mkdtemp(prefix="gnt-capa-"))
    edge = subprocess.Popen([str(EDGE), "--kiosk", PDF.as_uri(), "--edge-kiosk-type=fullscreen", "--window-position=0,0",
                             f"--user-data-dir={tmp / 'edge'}", "--no-first-run", "--no-default-browser-check"])
    time.sleep(5)
    a = None
    try:
        ctypes.windll.user32.SetProcessDPIAware()
        if not janela_por_cima():
            raise SystemExit("A janela do PDF não apareceu. Nada foi gravado.")
        time.sleep(1.5)
        ref = ImageGrab.grab()
        a = Anotador(GNT / "target" / "release" / "GizNaTela.exe", tmp / "dados", tmp / "capturas", "agir")
        L, A = a.estado()["tela"]["largura"], a.estado()["tela"]["altura"]
        X = lambda f: round(f * L)
        Y = lambda f: round(f * A)
        for passo in PASSOS(X, Y):
            a.valor(passo[0], **passo[1])
        a.valor("cor.escolher", cor="#e53935")
        time.sleep(6.5)   # o aviso "Agente: ..." some sozinho
        img = ImageGrab.grab()
        # a mesma janela tem de estar na frente: o canto de cima à esquerda (barra do leitor de PDF) não muda
        px = lambda im: im.getpixel((5, 5))[:3]
        if max(abs(x - y) for x, y in zip(px(img), px(ref))) > 8:
            raise SystemExit("Outra janela ficou na frente. Nada foi gravado.")
        # recorte sem o espaço vazio de baixo nem o selo do leitor de PDF no canto (marca de terceiro)
        k = img.size[0] / 1920
        img.crop((round(300 * k), 0, img.size[0], round(911 * k))).save(IMG / "capa-foto.png", optimize=True)
        print("foto gravada:", img.size)
        return 0
    finally:
        if a:
            a.fechar()
        edge.terminate()
        time.sleep(1)
        shutil.rmtree(tmp, ignore_errors=True)


def PASSOS(X, Y):
    # posições em pixels da foto de referência (1920 x 1080), convertidas em frações da tela
    x = lambda p: X(p / 1920)
    y = lambda p: Y(p / 1080)
    return [
        ("traco.desenhar", dict(pontos=[[x(474), y(311)], [x(652), y(311)]], marca=True, cor="#f9c80e", espessura="grossa")),
        ("numero.carimbar", dict(x=x(452), y=y(254), cor="#1e6fd9")),
        ("numero.carimbar", dict(x=x(452), y=y(551), cor="#1e6fd9")),
        ("traco.desenhar", dict(pontos=circulo(x(722), y(406), x(88), y(34)), cor="#e53935")),
        ("forma.desenhar", dict(forma="seta", x1=x(815), y1=y(398), x2=x(1165), y2=y(428), cor="#e53935")),
        ("texto.escrever", dict(texto="a inclinação da reta\né a resistência: 20 Ω", x=x(1100), y=y(605), cor="#e53935")),
        ("traco.desenhar", dict(pontos=circulo(x(1288), y(343), x(24), y(24)), cor="#1e6fd9")),
        ("texto.escrever", dict(texto="0,6 A e 12 V", x=x(1310), y=y(365), cor="#1e6fd9")),
        ("correcao.carimbar", dict(marca="visto", x=x(712), y=y(636), cor="#2e9d4a")),
    ]


if __name__ == "__main__":
    sys.exit(main())

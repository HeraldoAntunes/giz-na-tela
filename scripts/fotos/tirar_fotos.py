# Giz na Tela — © 2026 Heraldo Antunes — Licença MIT (ver LICENSE). Tira as fotos de uso do README
"""Abre o slide de exemplo (slide.html) em tela cheia no Edge, com perfil temporário, põe o Giz na Tela por cima
(pastas temporárias, agentes em "agir") e anota pelos mesmos comandos dos botões, pela API local. Cada foto é a tela
inteira, com a barra. Precisa do Pillow (só para fotografar). A tela fica ocupada por cerca de 1 minuto.
Uso: python scripts/fotos/tirar_fotos.py [--so-referencia]"""
import ctypes
import json
import math
import shutil
import subprocess
import sys
import tempfile
import time
from pathlib import Path

from PIL import ImageGrab

RAIZ = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(RAIZ / "tests"))
from testar_comandos import Anotador  # noqa: E402

SAIDA = RAIZ / "docs" / "img"
EDGE = Path(r"C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe")


PRONTAS = {}   # as fotos só vão para docs/img no fim, e só se todas passarem na trava


def slide_na_frente(img):
    """A borda esquerda do slide (fundo #f4f6f8) tem de estar na foto: se outra janela ficou na frente, a foto
    mostraria o que o professor tem aberto no computador, e nada é gravado."""
    w, h = img.size
    return all(max(abs(c - r) for c, r in zip(img.getpixel((round(w * 0.012), round(h * f)))[:3], (244, 246, 248))) <= 6
               for f in (0.3, 0.5, 0.7))


def foto(nome, conferir=True):
    img = ImageGrab.grab()
    if conferir and not slide_na_frente(img):
        raise SystemExit(f"A foto {nome} não mostra o slide (outra janela ficou na frente). Nenhuma foto foi gravada.")
    PRONTAS[nome] = img
    print("foto:", nome, img.size)
    return img


def slide_por_cima():
    """Põe a janela do slide "sempre por cima" (o Windows não deixa um programa roubar o foco de quem está usando o
    computador; "sempre por cima" não depende do foco). O Giz na Tela, aberto depois, fica acima dela."""
    u = ctypes.windll.user32
    achadas = []

    @ctypes.WINFUNCTYPE(ctypes.c_bool, ctypes.c_void_p, ctypes.c_void_p)
    def cada(h, _):
        n = u.GetWindowTextLengthW(h)
        b = ctypes.create_unicode_buffer(n + 1)
        u.GetWindowTextW(h, b, n + 1)
        if b.value.startswith("Plano inclinado") and u.IsWindowVisible(h):
            achadas.append(h)
        return True

    u.EnumWindows(cada, None)
    for h in achadas:
        u.SetWindowPos(ctypes.c_void_p(h), ctypes.c_void_p(-1), 0, 0, 0, 0, 0x0001 | 0x0002 | 0x0040)
    return bool(achadas)


def circulo(cx, cy, rx, ry, voltas=1.08, n=70):
    """Contorno feito à mão: o raio oscila um pouco e a pressão cresce e diminui, como a caneta de verdade."""
    pts = []
    for i in range(n + 1):
        t = i / n * voltas * 2 * math.pi - 0.6
        f = 1 + 0.04 * math.sin(3 * t)
        pts.append([cx + rx * f * math.cos(t), cy + ry * f * math.sin(t), round(0.35 + 0.5 * math.sin(math.pi * i / n), 2)])
    return pts


def main():
    tmp = Path(tempfile.mkdtemp(prefix="giz-na-tela-fotos-"))
    edge = subprocess.Popen([str(EDGE), "--kiosk", (RAIZ / "scripts" / "fotos" / "slide.html").as_uri(),
                             "--edge-kiosk-type=fullscreen", "--window-position=0,0",   # a tela principal, a que o Giz na Tela cobre
                             f"--user-data-dir={tmp / 'edge'}", "--no-first-run",
                             "--no-default-browser-check"])
    time.sleep(4)
    a = None
    try:
        ctypes.windll.user32.SetProcessDPIAware()
        if not slide_por_cima():
            raise SystemExit("A janela do slide não apareceu. Nenhuma foto foi gravada.")
        time.sleep(1)
        if not slide_na_frente(ImageGrab.grab()):
            raise SystemExit("O slide não ficou na frente. Nenhuma foto foi gravada.")
        if "--so-referencia" in sys.argv:
            foto("_referencia")
            PRONTAS["_referencia"].save(SAIDA / "_referencia.png")
            return 0
        a = Anotador(RAIZ / "target" / "release" / "GizNaTela.exe", tmp / "dados", tmp / "capturas", "agir")
        L, A = a.estado()["tela"]["largura"], a.estado()["tela"]["altura"]
        X = lambda f: round(f * L)
        Y = lambda f: round(f * A)
        # 1) slide anotado: marca-texto no título, círculo na aceleração, seta e números nas forças, visto
        a.valor("traco.desenhar", pontos=[[X(.088), Y(.13)], [X(.437), Y(.13)]], marca=True, cor="#f9c80e",
                espessura="grossa")
        for y in (.329, .381, .437):
            a.valor("numero.carimbar", x=X(.072), y=Y(y), cor="#1e6fd9")
        a.valor("traco.desenhar", pontos=circulo(X(.20), Y(.551), X(.112), Y(.04)), cor="#e53935")
        a.valor("correcao.carimbar", marca="visto", x=X(.33), y=Y(.548), cor="#2e9d4a")
        a.valor("forma.desenhar", forma="seta", x1=X(.36), y1=Y(.60), x2=X(.725), y2=Y(.545), cor="#e53935")
        a.valor("texto.escrever", texto="só a componente do peso\nao longo do plano acelera", x=X(.18), y=Y(.64),
                cor="#e53935")
        a.valor("cor.escolher", cor="#e53935")
        time.sleep(6.5)   # o aviso "Agente: ..." some sozinho
        foto("uso-slide")
        # 2) barra recolhida: o mouse fica livre para passar o slide e a tinta continua
        a.valor("barra.recolher", recolhida=True)
        time.sleep(6.5)
        foto("uso-recolhida")
        a.valor("barra.recolher", recolhida=False)
        # 3) holofote: escurece tudo em volta do ponto que interessa
        a.valor("desenho.limpar")
        a.valor("ferramenta.escolher", ferramenta="holofote")
        # o holofote segue o ponteiro: leva o mouse até o bloco do plano inclinado (px físicos da tela)
        k = ImageGrab.grab().size[0] / L
        ctypes.windll.user32.SetCursorPos(round(X(.735) * k), round(Y(.42) * k))
        time.sleep(6.5)
        foto("uso-holofote", conferir=False)   # o holofote escurece a borda
        a.valor("ferramenta.escolher", ferramenta="caneta")
        # 4) quadro negro por cima de tudo, para resolver uma conta
        a.valor("quadro.alternar", quadro="negro")
        a.valor("cor.escolher", cor="#ffffff")
        a.valor("texto.escrever", texto="m = 2 kg   θ = 30°   g = 10 m/s²", x=X(.08), y=Y(.12), tamanho=44)
        a.valor("texto.escrever", texto="a = g · sen θ\na = 10 · 0,5\na = 5 m/s²", x=X(.08), y=Y(.25), tamanho=44)
        a.valor("traco.desenhar", pontos=circulo(X(.135), Y(.371), X(.08), Y(.034)), cor="#f9c80e", espessura="grossa")
        a.valor("forma.desenhar", forma="retangulo", x1=X(.55), y1=Y(.18), x2=X(.80), y2=Y(.36), cor="#f9c80e")
        a.valor("texto.escrever", texto="Confira:\nsen 30° = 0,5", x=X(.57), y=Y(.22), tamanho=40, cor="#f9c80e")
        a.valor("cor.escolher", cor="#ffffff")
        time.sleep(6.5)
        foto("uso-quadro-negro", conferir=False)   # o quadro cobre o slide inteiro
        a.valor("quadro.alternar", quadro="tela")
        a.valor("desenho.limpar")
        # 5) ajuda e ajustes, com a barra ao lado
        a.valor("ajuda.abrir")
        time.sleep(6.5)
        foto("uso-ajuda")
        for nome, img in PRONTAS.items():
            img.save(SAIDA / f"{nome}.png", optimize=True)
        print("gravadas em docs/img:", ", ".join(PRONTAS))
        return 0
    finally:
        if a:
            a.fechar()
        edge.terminate()
        time.sleep(1)
        shutil.rmtree(tmp, ignore_errors=True)


if __name__ == "__main__":
    sys.exit(main())

# Giz na Tela — © 2026 Heraldo Antunes — Licença MIT (ver LICENSE). Teste de ponta a ponta (só biblioteca padrão)
"""Abre o GizNaTela.exe com pastas temporárias (nunca as do professor), chama cada comando do registro pela API local
(o mesmo caminho do agente de IA, que é o mesmo do clique) e confere o efeito, inclusive pixels do retrato da tela.
Confere também a segurança da API, os níveis dos agentes, o MCP e a lembrança da posição da barra.

Uso: python tests/testar_comandos.py [--exe target/release/GizNaTela.exe] [--giz CAMINHO/server.py] [--retrato PASTA]
  --giz      sobe um Giz Livre de desenvolvimento (porta livre, dados temporários) para testar o envio ao Giz
  --retrato  guarda retratos da tela de cada etapa, para a auditoria visual
A camada cobre a tela por alguns segundos durante o teste (com quadro branco, para não retratar o que estiver aberto).
"""
import argparse
import base64
import ctypes
import json
import os
import shutil
import socket
import struct
import subprocess
import sys
import tempfile
import time
import urllib.error
import urllib.request
import zlib
from pathlib import Path

RAIZ = Path(__file__).resolve().parent.parent
ok_total, falhas = 0, []


def confere(cond, msg):
    global ok_total
    if cond:
        ok_total += 1
        print("  ok  ", msg)
    else:
        falhas.append(msg)
        print("  FALHA", msg)


def porta_livre():
    with socket.socket() as s:
        s.bind(("127.0.0.1", 0))
        return s.getsockname()[1]


# ---------- PNG mínimo (RGB/RGBA de 8 bits) para conferir pixels do retrato ----------
def ler_png(dados):
    assert dados[:8] == b"\x89PNG\r\n\x1a\n"
    i, idat, larg = 8, b"", 0
    while i < len(dados):
        n, tipo = struct.unpack(">I4s", dados[i:i + 8])
        corpo = dados[i + 8:i + 8 + n]
        if tipo == b"IHDR":
            larg, alt, prof, cor = struct.unpack(">IIBB", corpo[:10])
            assert prof == 8 and cor in (2, 6)
            bpp = 4 if cor == 6 else 3
        elif tipo == b"IDAT":
            idat += corpo
        i += 12 + n
    cru, linha, ant, img = zlib.decompress(idat), larg * bpp, bytearray(larg * bpp), []
    for y in range(alt):
        f, lin = cru[y * (linha + 1)], bytearray(cru[y * (linha + 1) + 1:(y + 1) * (linha + 1)])
        for x in range(linha):
            a = lin[x - bpp] if x >= bpp else 0
            b, c = ant[x], ant[x - bpp] if x >= bpp else 0
            if f == 1: lin[x] = (lin[x] + a) & 255
            elif f == 2: lin[x] = (lin[x] + b) & 255
            elif f == 3: lin[x] = (lin[x] + (a + b) // 2) & 255
            elif f == 4:
                p = a + b - c; pa, pb, pc = abs(p - a), abs(p - b), abs(p - c)
                lin[x] = (lin[x] + (a if pa <= pb and pa <= pc else b if pb <= pc else c)) & 255
        img.append(bytes(lin)); ant = lin
    return larg, alt, bpp, img


def pixel(png, x, y):
    larg, alt, bpp, img = png
    return tuple(img[int(y)][int(x) * bpp:int(x) * bpp + 3])


def perto(c1, c2, tol=40):
    return all(abs(a - b) <= tol for a, b in zip(c1, c2))


# ---------- o programa ----------
class Anotador:
    def __init__(self, exe, dados, capturas, nivel="agir", porta_giz=None, config_extra=None):
        self.exe, self.dados, self.porta = exe, Path(dados), porta_livre()
        cfg = {"agentes": nivel, **(config_extra or {})}
        self.dados.mkdir(parents=True, exist_ok=True)
        atual = {}
        if (self.dados / "config.json").exists():
            atual = json.loads((self.dados / "config.json").read_text(encoding="utf-8"))
        (self.dados / "config.json").write_text(json.dumps({**atual, **cfg}), encoding="utf-8")
        (self.dados / "agente" / "porta").unlink(missing_ok=True)
        args = [str(exe), "--dados", str(dados), "--capturas", str(capturas), "--porta-api", str(self.porta)]
        if porta_giz:
            args += ["--porta-giz", str(porta_giz)]
        self.proc = subprocess.Popen(args)
        for _ in range(100):
            if (self.dados / "agente" / "porta").exists():
                break
            time.sleep(0.1)
        time.sleep(1.5)   # as páginas da barra e da camada terminam de carregar
        self.token = (self.dados / "agente" / "token").read_text().strip() if (self.dados / "agente" / "token").exists() else ""

    def http(self, metodo, rota, corpo=None, cab=None, cru=None):
        dados = cru if cru is not None else (json.dumps(corpo).encode() if corpo is not None else None)
        h = {"X-Giz-Tela": self.token, "Content-Type": "application/json"}
        h.update(cab or {})
        req = urllib.request.Request(f"http://127.0.0.1:{self.porta}{rota}", data=dados, method=metodo, headers=h)
        try:
            with urllib.request.urlopen(req, timeout=90) as r:
                return r.status, json.loads(r.read())
        except urllib.error.HTTPError as e:
            with e:
                return e.code, json.loads(e.read() or b"{}")

    def cmd(self, nome, **p):
        return self.http("POST", "/api/comando", {"nome": nome, "parametros": p})

    def valor(self, nome, **p):
        s, r = self.cmd(nome, **p)
        if s != 200:
            raise AssertionError(f"{nome} {p}: {s} {r}")
        return r["valor"]

    def estado(self):
        return self.valor("estado.ler")

    def retrato(self, nome=None):
        v = self.valor("captura.ler", largura_max=3840)
        dados = base64.b64decode(v["png_base64"])
        if nome and RETRATOS:
            (RETRATOS / f"{nome}.png").write_bytes(dados)
        return ler_png(dados)

    def fechar(self):
        if self.proc.poll() is None:
            try:
                self.cmd("programa.fechar", confirmar=True)
                self.proc.wait(10)
            except Exception:
                self.proc.kill()


# ---------- um teste por comando ----------
def t_ferramenta(a):
    for f in ("marca", "seta", "holofote", "caneta"):
        a.valor("ferramenta.escolher", ferramenta=f)
        confere(a.estado()["ferramenta"] == f, f"ferramenta.escolher {f}")
    a.valor("ferramenta.escolher", ferramenta="lupa")
    confere(a.estado()["congelado"], "ferramenta.escolher lupa congela a tela sozinha")
    a.valor("ferramenta.escolher", ferramenta="caneta")
    confere(not a.estado()["congelado"], "sair da lupa descongela")
    s, r = a.cmd("ferramenta.escolher", ferramenta="lapis")
    confere(s == 400 and "use" in r["erro"], "ferramenta.escolher recusa ferramenta inexistente")


def t_cor(a):
    a.valor("cor.escolher", cor="#1E6FD9")
    confere(a.estado()["cor"] == "#1e6fd9", "cor.escolher")
    a.valor("ferramenta.escolher", ferramenta="borracha")
    a.valor("cor.escolher", cor="#e53935")
    confere(a.estado()["ferramenta"] == "caneta", "cor.escolher com a borracha volta para a caneta")
    confere(a.cmd("cor.escolher", cor="vermelho")[0] == 400, "cor.escolher recusa cor fora de #rrggbb")


def t_espessura(a):
    a.valor("espessura.escolher", espessura="grossa")
    confere(a.estado()["espessura"] == "grossa", "espessura.escolher")
    a.valor("espessura.escolher", espessura="media")


def t_desfazer_refazer(a):
    n = a.estado()["desenhos"]
    a.valor("texto.escrever", texto="desfazer", x=100, y=100)
    confere(a.valor("desenho.desfazer")["desenhos"] == n, "desenho.desfazer")
    confere(a.valor("desenho.refazer")["desenhos"] == n + 1, "desenho.refazer")


def t_limpar(a):
    a.valor("texto.escrever", texto="limpar", x=100, y=140)
    e = a.valor("desenho.limpar")
    confere(e["desenhos"] == 0 and e["pode_desfazer"], "desenho.limpar")
    confere(a.valor("desenho.desfazer")["desenhos"] > 0, "desenho.limpar se desfaz")
    a.valor("desenho.limpar")


def t_quadro(a):
    a.valor("quadro.alternar", quadro="tela")
    seq =[a.valor("quadro.alternar")["quadro"] for _ in range(3)]
    confere(seq == ["branco", "negro", "tela"], f"quadro.alternar dá a volta ({seq})")
    a.valor("quadro.alternar", quadro="negro")
    p = a.retrato("quadro-negro")
    confere(perto(pixel(p, p[0] // 3, p[1] // 3), (0x1d, 0x2b, 0x24)), "quadro.alternar negro cobre a tela")
    a.valor("quadro.alternar", quadro="branco")


def t_preencher(a):
    seq = [a.valor("forma.preencher")["preencher"] for _ in range(3)]
    confere(seq == ["translucido", "cheio", "nada"], f"forma.preencher dá a volta ({seq})")


def t_seta(a):
    confere(a.estado()["ponta_da_seta"] == "aberta", "seta.estilo: ponta aberta de fábrica")
    confere(a.valor("seta.estilo")["ponta_da_seta"] == "cheia", "seta.estilo sem parâmetro alterna")
    confere(a.valor("seta.estilo", estilo="aberta")["ponta_da_seta"] == "aberta", "seta.estilo aberta")
    confere(a.cmd("seta.estilo", estilo="dupla")[0] == 400, "seta.estilo recusa estilo inexistente")
    tipos = []
    for _ in range(4):
        e = a.valor("seta.estilo")
        tipos.append((e["ponta_da_seta"], e["seta_dupla"]))
    confere(tipos == [("cheia", False), ("aberta", True), ("cheia", True), ("aberta", False)],
            f"seta.estilo passa pelos 4 tipos na ordem ({tipos})")
    a.valor("desenho.limpar")
    a.valor("forma.desenhar", forma="seta", x1=300, y1=500, x2=600, y2=500, cor="#111111", espessura="grossa")
    a.valor("forma.desenhar", forma="seta", x1=300, y1=600, x2=600, y2=600, cor="#111111", espessura="grossa", ponta="cheia")
    p = a.retrato("setas")
    k = p[0] / a.estado()["tela"]["largura"]
    # 24 px antes da ponta, a 6 px da linha: a ponta cheia é triângulo pintado; a aberta é só o contorno em V
    confere(not perto(pixel(p, 576 * k, 606 * k), (255, 255, 255), 60), "ponta cheia pinta o meio do triângulo")
    confere(perto(pixel(p, 576 * k, 506 * k), (255, 255, 255), 60), "ponta aberta deixa o meio do V vazio")
    a.valor("desenho.limpar")


def t_cota(a):
    seq = [a.valor("cota.estilo")["tipo_de_cota"] for _ in range(2)]
    confere(seq == ["tracos", "setas"], f"cota.estilo alterna setas e tracinhos ({seq})")
    confere(a.cmd("cota.estilo", estilo="dupla")[0] == 400, "cota.estilo recusa tipo inexistente")
    a.valor("desenho.limpar")
    a.valor("forma.desenhar", forma="cota", x1=300, y1=700, x2=800, y2=700, cor="#1e6fd9", espessura="grossa",
            medida="2,5 m")
    a.valor("forma.desenhar", forma="cota", x1=300, y1=850, x2=800, y2=850, cor="#1e6fd9", cota="tracos")
    p = a.retrato("cotas")
    k = p[0] / a.estado()["tela"]["largura"]
    azul = lambda x, y: not perto(pixel(p, x * k, y * k), (255, 255, 255), 60)
    confere(azul(300, 690) and azul(800, 712), "cota: barras perpendiculares nas duas pontas")
    confere(any(azul(x, y) for x in range(530, 571, 3) for y in (694, 700, 706)),
            "cota com medida: o texto no meio da linha")
    confere(a.estado()["desenhos"] == 2, "forma.desenhar cota (com medida e de arquitetura)")
    confere(a.cmd("forma.desenhar", forma="cota", x1=1, y1=1, x2=9, y2=9, medida="")[0] == 400,
            "cota recusa medida em branco")
    a.valor("desenho.limpar")


def t_cores(a):
    a.valor("cores.restaurar")
    confere(a.valor("cor.posicao", posicao=1)["cor"] == "#2e9d4a", "cor.posicao 1 é o verde de fábrica")
    confere(a.valor("cor.posicao", posicao=2)["cor"] == "#e53935", "cor.posicao 2 é o vermelho")
    e = a.valor("cor.trocar", posicao=3, cor="#7B1FA2")
    confere(e["cores"][2] == "#7b1fa2" and e["cor"] == "#7b1fa2", "cor.trocar troca a bolinha e já a escolhe")
    s_, r = a.cmd("cor.trocar", posicao=3)
    confere(s_ == 400 and "mande a cor" in r["erro"], "cor.trocar de fora sem cor é recusado (não abre seletor)")
    confere(a.cmd("cor.posicao", posicao=7)[0] == 400, "cor.posicao recusa bolinha inexistente")
    e = a.valor("cores.restaurar")
    confere(e["cores"][2] == "#1e6fd9" and e["cor"] == "#2e9d4a", "cores.restaurar volta às de fábrica")


def t_escala(a):
    a.valor("desenho.limpar")
    a.valor("cota.escala")
    confere(a.estado()["regua"] is None, "cota.escala sem parâmetros tira a régua")
    a.valor("forma.desenhar", forma="cota", cota="auto", x1=300, y1=400, x2=550, y2=400)
    confere(a.estado()["ultima_medida"] == "250 px", f"cota automática sem régua mede em px ({a.estado()['ultima_medida']})")
    a.valor("cota.escala", valor=10, unidade="cm", pixels=200)
    a.valor("forma.desenhar", forma="cota", cota="auto", x1=300, y1=500, x2=600, y2=900)
    confere(a.estado()["ultima_medida"] == "25 cm", f"cota automática com régua mede em cm ({a.estado()['ultima_medida']})")
    a.valor("forma.desenhar", forma="cota", cota="auto", x1=300, y1=600, x2=313, y2=600)
    confere(a.estado()["ultima_medida"] == "0,65 cm", f"medida pequena com vírgula ({a.estado()['ultima_medida']})")
    a.retrato("cota-auto")
    a.valor("cota.escala")
    a.valor("desenho.limpar")


def t_mover(a):
    a.valor("desenho.limpar")
    a.valor("forma.desenhar", forma="retangulo", x1=300, y1=300, x2=400, y2=380, cor="#1e6fd9", preencher="cheio")
    a.valor("texto.escrever", texto="mover", x=600, y=300, tamanho=40)
    confere(a.cmd("objeto.mover", x=900, y=700, dx=10, dy=10)[0] == 400, "objeto.mover num ponto vazio é recusado")
    a.valor("objeto.mover", x=350, y=340, dx=300, dy=200)   # pega o retângulo pelo meio (cheio) e leva
    p = a.retrato("mover")
    k = p[0] / a.estado()["tela"]["largura"]
    confere(perto(pixel(p, 650 * k, 540 * k), (0x1e, 0x6f, 0xd9)) and perto(pixel(p, 350 * k, 340 * k), (255, 255, 255)),
            "objeto.mover leva o retângulo para o lugar novo")
    a.valor("objeto.mover", x=620, y=320, dx=0, dy=150)   # o texto
    confere(a.estado()["desenhos"] == 2, "mover não cria nem apaga objetos")
    a.valor("desenho.desfazer"); a.valor("desenho.desfazer")
    p = a.retrato()
    confere(perto(pixel(p, 350 * k, 340 * k), (0x1e, 0x6f, 0xd9)), "Ctrl+Z devolve o objeto ao lugar, um passo por movimento")
    a.valor("desenho.limpar")


def t_some(a):
    a.valor("tinta.some", ligado=True)
    n = a.estado()["desenhos"]
    a.valor("texto.escrever", texto="some", x=300, y=300)
    confere(a.estado()["desenhos"] == n + 1, "tinta.some: o texto aparece")
    time.sleep(3.6)
    confere(a.estado()["desenhos"] == n, "tinta.some: e some sozinho em 3 s")
    a.valor("tinta.some", ligado=False)
    confere(not a.estado()["tinta_some"], "tinta.some desliga")


def t_congelar(a):
    confere(a.valor("tela.congelar", ligado=True)["congelado"], "tela.congelar liga")
    confere(not a.valor("tela.congelar")["congelado"], "tela.congelar sem parâmetro alterna")


def t_ampliar(a):
    confere(a.valor("tela.ampliar", modo="ligar")["ampliado"], "tela.ampliar ligar")
    a.valor("tela.ampliar", modo="mais")
    e = a.valor("tela.ampliar", modo="desligar")
    confere(not e["ampliado"] and not e["congelado"], "tela.ampliar desligar descongela")


def t_fixar(a):
    confere(a.valor("tela.fixar", ligado=True)["modo"] == "mouse livre", "tela.fixar: mouse livre")
    a.valor("texto.escrever", texto="agente com mouse livre", x=120, y=600)
    confere(a.estado()["modo"] == "mouse livre", "agente desenhando não muda o modo do professor")
    confere(a.valor("tela.fixar")["modo"] == "escrevendo", "tela.fixar alterna de volta")
    a.valor("desenho.limpar")


def t_recolher(a):
    e = a.valor("barra.recolher", recolhida=True)
    confere(e["barra_recolhida"] and e["modo"] == "mouse livre", "barra.recolher: botão pequeno e mouse livre")
    time.sleep(1.5)
    cfg = json.loads((a.dados / "config.json").read_text(encoding="utf-8"))
    confere(cfg.get("recolhida") is True and cfg.get("barra"), "barra.recolher fica gravado (estado e posição)")
    e = a.valor("barra.recolher", recolhida=False)
    confere(not e["barra_recolhida"] and e["modo"] == "escrevendo", "barra.recolher false abre e volta a escrever")


def t_salvar(a, capturas):
    a.valor("forma.desenhar", forma="retangulo", x1=200, y1=200, x2=320, y2=300, cor="#2e9d4a", preencher="cheio")
    r = a.valor("captura.salvar")
    arq = Path(r["arquivo"])
    confere(arq.exists() and arq.parent == Path(capturas), "captura.salvar grava na pasta de capturas")
    p = ler_png(arq.read_bytes())
    k = p[0] / a.estado()["tela"]["largura"]
    confere(perto(pixel(p, 260 * k, 250 * k), (0x2e, 0x9d, 0x4a)), "captura.salvar: a tinta está na imagem")
    a.valor("desenho.limpar")


def t_giz(a, giz_info):
    a.valor("texto.escrever", texto="para o Giz", x=200, y=200)
    r = a.valor("captura.mandar_ao_giz")
    if giz_info is None:
        confere(r["giz"] is False and "Giz Livre fechado" in r["mensagem"] and Path(r["arquivo"]).exists(),
                "captura.mandar_ao_giz com o Giz fechado salva a imagem e avisa (sem abrir o Giz do professor)")
        a.valor("desenho.limpar")
        return
    porta, token = giz_info
    req = urllib.request.Request(f"http://127.0.0.1:{porta}/api/boards", headers={"X-Lousa": token})
    quadros = json.load(urllib.request.urlopen(req))
    e = a.estado()
    confere(r["giz"] and any(q["title"] == r["quadro"] for q in quadros), "captura.mandar_ao_giz cria o quadro no Giz")
    confere(e["desenhos"] == 0 and e["barra_recolhida"], "depois de mandar: a tinta sai e a barra recolhe")
    a.valor("barra.recolher", recolhida=False)
    confere(a.valor("desenho.desfazer")["desenhos"] == 1, "Ctrl+Z traz a tinta mandada ao Giz de volta")
    a.valor("desenho.limpar")


def t_ler(a):
    v = a.valor("captura.ler", largura_max=640)
    confere(v["largura"] <= 640 and base64.b64decode(v["png_base64"])[:4] == b"\x89PNG", "captura.ler reduz e devolve PNG")


def t_texto(a):
    n = a.estado()["desenhos"]
    confere(a.valor("texto.escrever", texto="Olá\nsegunda linha", x=400, y=120, tamanho=40)["desenhos"] == n + 1, "texto.escrever")
    confere(a.cmd("texto.escrever", texto="   ", x=1, y=1)[0] == 400, "texto.escrever recusa texto em branco")
    confere(a.cmd("texto.escrever", texto="a", x=1, y=1, fonte="Arial")[0] == 400, "parâmetro desconhecido é recusado")
    confere(a.cmd("texto.escrever", texto="a")[0] == 400, "parâmetro obrigatório faltando é recusado")


def t_numero(a):
    a.valor("desenho.limpar")
    a.valor("numero.carimbar", x=100, y=400)
    a.valor("numero.carimbar", x=160, y=400, numero=7)
    a.valor("numero.carimbar", x=220, y=400)
    p = a.retrato("numeros")
    k = p[0] / a.estado()["tela"]["largura"]
    confere(not perto(pixel(p, 100 * k, 400 * k - 13 * k), (255, 255, 255), 10), "numero.carimbar desenha o círculo")
    confere(a.estado()["desenhos"] == 3, "numero.carimbar com e sem número")


def t_correcao(a):
    n = a.estado()["desenhos"]
    a.valor("correcao.carimbar", marca="visto", x=300, y=400, cor="#2e9d4a")
    a.valor("correcao.carimbar", marca="xis", x=360, y=400)
    confere(a.estado()["desenhos"] == n + 2, "correcao.carimbar visto e xis")
    confere(a.cmd("correcao.carimbar", marca="certo", x=1, y=1)[0] == 400, "correcao.carimbar recusa marca inexistente")


def t_forma(a):
    a.valor("desenho.limpar")
    for f, y in (("reta", 500), ("seta", 560), ("elipse", 620)):
        a.valor("forma.desenhar", forma=f, x1=500, y1=y, x2=700, y2=y + 40)
    a.valor("forma.desenhar", forma="retangulo", x1=760, y1=500, x2=900, y2=600, cor="#1e6fd9", preencher="cheio")
    p = a.retrato("formas")
    k = p[0] / a.estado()["tela"]["largura"]
    confere(perto(pixel(p, 830 * k, 550 * k), (0x1e, 0x6f, 0xd9)), "forma.desenhar retângulo cheio no lugar certo")
    confere(perto(pixel(p, 1000 * k, 550 * k), (255, 255, 255)), "forma.desenhar não pinta fora da forma")


def t_traco(a):
    n = a.estado()["desenhos"]
    pts = [[100 + i * 6, 700 + (i % 10) * 3, i / 60] for i in range(60)]
    a.valor("traco.desenhar", pontos=pts, cor="#111111", espessura="grossa")
    a.valor("traco.desenhar", pontos=[[100, 760], [400, 760]], marca=True, cor="#f9c80e")
    confere(a.estado()["desenhos"] == n + 2, "traco.desenhar (caneta e marca-texto)")
    confere(a.cmd("traco.desenhar", pontos=[[1, 2, 3, 4]])[0] == 400, "traco.desenhar recusa ponto malformado")


def t_adicionar(a):
    a.valor("desenho.limpar")
    itens = [{"tipo": "texto", "texto": "Agente", "x": 1100, "y": 120},
             {"tipo": "forma", "forma": "seta", "x1": 1100, "y1": 200, "x2": 1250, "y2": 300},
             {"tipo": "numero", "x": 1300, "y": 320},
             {"tipo": "correcao", "marca": "visto", "x": 1350, "y": 320},
             {"tipo": "traco", "pontos": [[1100, 400], [1200, 420], [1300, 400]]}]
    confere(a.valor("desenho.adicionar", itens=itens)["desenhos"] == 5, "desenho.adicionar põe 5 itens")
    a.retrato("agente")
    confere(a.valor("desenho.desfazer")["desenhos"] == 0, "desenho.adicionar se desfaz num passo só")
    ruim = itens[:2] + [{"tipo": "forma", "forma": "losango", "x1": 0, "y1": 0, "x2": 1, "y2": 1}]
    s, r = a.cmd("desenho.adicionar", itens=ruim)
    confere(s == 400 and "itens[2]" in r["erro"] and a.estado()["desenhos"] == 0,
            "desenho.adicionar com um item ruim não põe nenhum e diz qual falhou")


def t_estado(a):
    e = a.estado()
    faltam = {"versao", "modo", "ferramenta", "cor", "espessura", "quadro", "desenhos", "tela"} - set(e)
    confere(not faltam and e["tela"]["largura"] > 0, f"estado.ler completo (faltam: {faltam or 'nada'})")


def t_listar(a, lista):
    confere(lista["problemas"] == [], f"comandos.listar sem problemas de registro: {lista['problemas']}")
    confere(all(c["descricao"] for c in lista["comandos"]), "todo comando tem descrição")
    confere(len(lista["botoes"]) >= 35, f"botões da barra ligados ao registro ({len(lista['botoes'])})")


def t_voltar(a):
    a.valor("ferramenta.escolher", ferramenta="seta")
    a.valor("quadro.alternar", quadro="negro")
    a.valor("tela.congelar", ligado=True)
    a.valor("tela.ampliar", modo="ligar")
    passos = []
    for _ in range(5):
        e = a.valor("modo.voltar_um_passo") and a.estado()
        passos.append((e["ampliado"], e["congelado"], e["quadro"], e["ferramenta"], e["modo"]))
    esperado = [(False, True, "negro", "seta", "escrevendo"), (False, False, "negro", "seta", "escrevendo"),
                (False, False, "tela", "seta", "escrevendo"), (False, False, "tela", "caneta", "escrevendo"),
                (False, False, "tela", "caneta", "mouse livre")]
    confere(passos == esperado, f"modo.voltar_um_passo (Esc) na ordem certa: {passos}")
    confere(a.proc.poll() is None, "Esc nunca fecha o anotador")
    a.valor("tela.fixar", ligado=False)
    a.valor("quadro.alternar", quadro="branco")


def t_ajuda(a):
    s, r = a.cmd("ajuda.abrir")
    confere(s == 200, "ajuda.abrir")


def t_fechar(a):
    # escolhas que têm de voltar na próxima abertura (conferidas mais adiante, ao abrir de novo)
    a.valor("seta.estilo", estilo="cheia", dupla=True)
    a.valor("cota.estilo", estilo="tracos")
    a.valor("cor.trocar", posicao=2, cor="#123456")
    a.valor("espessura.escolher", espessura="grossa")
    a.valor("texto.escrever", texto="não fecha", x=100, y=100)
    s, r = a.cmd("programa.fechar")
    confere(s == 400 and "confirmar" in r["erro"] and a.proc.poll() is None, "programa.fechar com desenho pede confirmar")
    confere(a.valor("programa.fechar", confirmar=True)["fechou"], "programa.fechar confirmar: true")
    try:
        a.proc.wait(5)
    except subprocess.TimeoutExpired:
        pass
    confere(a.proc.poll() is not None, "o programa fechou")


def seguranca(a):
    base = f"http://127.0.0.1:{a.porta}/api/comandos"
    def pedir(cab, metodo="GET", url=base, dados=None):
        req = urllib.request.Request(url, headers=cab, method=metodo, data=dados)
        try:
            with urllib.request.urlopen(req, timeout=10) as r:
                return r.status
        except urllib.error.HTTPError as e:
            return e.code
        except ConnectionError:
            return 413   # recusou e fechou antes de o corpo inteiro chegar: também é recusa
    confere(pedir({"X-Giz-Tela": a.token}) == 200, "API com token responde")
    confere(pedir({}) == 401, "API sem token: 401")
    confere(pedir({"X-Giz-Tela": "0" * 64}) == 401, "API com token errado: 401")
    confere(pedir({"X-Giz-Tela": a.token, "Host": "exemplo.com"}) == 403, "API com Host de fora: 403 (DNS rebinding)")
    confere(pedir({"X-Giz-Tela": a.token, "Origin": "http://exemplo.com"}) == 403, "API chamada de página do navegador: 403")
    confere(pedir({"X-Giz-Tela": a.token, "Content-Type": "application/json"}, "POST",
                  base.replace("comandos", "comando"), b"x" * (1024 * 1024 + 10)) == 413, "API recusa corpo grande demais")
    confere(pedir({"X-Giz-Tela": a.token}, url=base.replace("comandos", "nada")) == 404, "API: rota desconhecida 404")
    confere(a.http("POST", "/api/comando", cru=b"{nao e json")[0] == 400, "API: JSON inválido 400")
    confere(a.cmd("nada.fazer")[0] == 400, "API: comando desconhecido 400")


def mcp(exe, dados):
    p = subprocess.Popen([str(exe), "--mcp", "--dados", str(dados)], stdin=subprocess.PIPE, stdout=subprocess.PIPE)
    n = 0
    def rpc(metodo, params=None):
        nonlocal n
        n += 1
        p.stdin.write((json.dumps({"jsonrpc": "2.0", "id": n, "method": metodo, "params": params or {}}) + "\n").encode())
        p.stdin.flush()
        return json.loads(p.stdout.readline())
    try:
        r = rpc("initialize", {"protocolVersion": "2025-06-18", "capabilities": {}, "clientInfo": {"name": "teste"}})
        confere(r["result"]["serverInfo"]["name"] == "giz-na-tela", "MCP initialize")
        p.stdin.write(b'{"jsonrpc":"2.0","method":"notifications/initialized"}\n')
        nomes = [t["name"] for t in rpc("tools/list")["result"]["tools"]]
        confere(nomes == ["estado_giz_na_tela", "listar_comandos", "capturar_tela", "desenhar", "executar_comando"],
                f"MCP tools/list ({nomes})")
        chama = lambda nome, args=None: rpc("tools/call", {"name": nome, "arguments": args or {}})["result"]
        e = json.loads(chama("estado_giz_na_tela")["content"][0]["text"])
        antes = e["desenhos"]
        r = chama("desenhar", {"itens": [{"tipo": "texto", "texto": "pelo MCP", "x": 600, "y": 200},
                                         {"tipo": "forma", "forma": "elipse", "x1": 580, "y1": 180, "x2": 800, "y2": 260}]})
        confere(not r.get("isError") and json.loads(r["content"][0]["text"])["desenhos"] == antes + 2, "MCP desenhar")
        r = chama("executar_comando", {"nome": "desenho.desfazer"})
        confere(json.loads(r["content"][0]["text"])["desenhos"] == antes, "MCP: o desenho do agente sai num Ctrl+Z")
        img = chama("capturar_tela", {"largura_max": 400})["content"][0]
        confere(img["type"] == "image" and base64.b64decode(img["data"])[:4] == b"\x89PNG", "MCP capturar_tela")
        r = chama("executar_comando", {"nome": "nao.existe"})
        confere(r.get("isError") and "desconhecido" in r["content"][0]["text"], "MCP: erro volta como isError")
        confere("comandos" in json.loads(chama("listar_comandos")["content"][0]["text"]), "MCP listar_comandos")
        confere(rpc("metodo/estranho")["error"]["code"] == -32601, "MCP: método desconhecido")
    finally:
        p.stdin.close()
        p.wait(5)


def mcp_desligado(exe, dados):
    p = subprocess.run([str(exe), "--mcp", "--dados", str(dados)], capture_output=True, timeout=20,
                       input=(json.dumps({"jsonrpc": "2.0", "id": 1, "method": "tools/call",
                                          "params": {"name": "estado_giz_na_tela", "arguments": {}}}) + "\n").encode())
    r = json.loads(p.stdout.decode().splitlines()[0])["result"]
    confere(r["isError"] and "desligados" in r["content"][0]["text"], "MCP com agentes desligados explica o que fazer")


def posicao_da_barra():
    """Retângulo da janela da barra (título "Giz na Tela"), em px físicos."""
    u = ctypes.windll.user32
    u.SetProcessDPIAware()
    h = u.FindWindowW(None, "Giz na Tela")
    r = (ctypes.c_long * 4)()
    u.GetWindowRect(h, r)
    return h, tuple(r)


def main():
    global RETRATOS
    ap = argparse.ArgumentParser()
    ap.add_argument("--exe", default=str(RAIZ / "target" / "release" / "GizNaTela.exe"))
    ap.add_argument("--giz", help="server.py do Giz Livre de desenvolvimento")
    ap.add_argument("--retrato", help="pasta para os retratos da auditoria visual")
    o = ap.parse_args()
    RETRATOS = Path(o.retrato) if o.retrato else None
    if RETRATOS:
        RETRATOS.mkdir(parents=True, exist_ok=True)
    exe = Path(o.exe)
    tmp = Path(tempfile.mkdtemp(prefix="giz-na-tela-teste-"))
    dados, capturas = tmp / "dados", tmp / "capturas"
    giz, giz_info, porta_giz = None, None, porta_livre()
    if o.giz:
        gdados = tmp / "giz"
        giz = subprocess.Popen([sys.executable, o.giz, "--sem-janela", "--porta", str(porta_giz), "--dados", str(gdados)],
                               stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        for _ in range(60):
            try:
                tok = json.load(urllib.request.urlopen(f"http://127.0.0.1:{porta_giz}/api/info", timeout=1))["token"]
                giz_info = (porta_giz, tok)
                break
            except OSError:
                time.sleep(0.25)
    try:
        print("== agentes em 'agir': um teste por comando")
        a = Anotador(exe, dados, capturas, "agir", porta_giz)
        try:
            lista = a.valor("comandos.listar")
            confere(a.estado()["cor"] == "#2e9d4a", "abre com o verde como cor de início")
            # a barra pode abrir depois da camada: mesmo assim já sabe o tamanho da tela
            confere(a.estado()["tela"]["largura"] > 0, "tamanho da tela conhecido logo ao abrir")
            a.valor("quadro.alternar", quadro="branco")   # fundo liso: o retrato não pega o que está aberto no PC
            testes = {
                "ferramenta.escolher": t_ferramenta, "cor.escolher": t_cor, "espessura.escolher": t_espessura,
                "desenho.desfazer": t_desfazer_refazer, "desenho.refazer": t_desfazer_refazer, "desenho.limpar": t_limpar,
                "quadro.alternar": t_quadro, "forma.preencher": t_preencher, "seta.estilo": t_seta, "cota.estilo": t_cota, "cor.posicao": t_cores, "cor.trocar": t_cores, "cores.restaurar": t_cores,
                "cota.escala": t_escala, "objeto.mover": t_mover, "tinta.some": t_some,
                "tela.congelar": t_congelar, "tela.ampliar": t_ampliar, "tela.fixar": t_fixar,
                "barra.recolher": t_recolher, "captura.salvar": lambda a: t_salvar(a, capturas),
                "captura.mandar_ao_giz": lambda a: t_giz(a, giz_info), "captura.ler": t_ler,
                "texto.escrever": t_texto, "numero.carimbar": t_numero, "correcao.carimbar": t_correcao,
                "forma.desenhar": t_forma, "traco.desenhar": t_traco, "desenho.adicionar": t_adicionar,
                "estado.ler": t_estado, "comandos.listar": lambda a: t_listar(a, lista),
                "modo.voltar_um_passo": t_voltar, "ajuda.abrir": t_ajuda, "programa.fechar": t_fechar,
            }
            nomes = [c["nome"] for c in lista["comandos"]]
            sem = [n for n in nomes if n not in testes]
            confere(not sem, f"todo comando do registro tem teste (sem teste: {sem or 'nenhum'})")
            seguranca(a)
            feitos = set()
            for nome in nomes:
                if nome in testes and testes[nome] not in feitos:
                    feitos.add(testes[nome])
                    try:
                        testes[nome](a)
                    except Exception as e:
                        confere(False, f"{nome}: {e}")
        finally:
            a.fechar()
        print("== MCP (com o anotador aberto)")
        a2 = Anotador(exe, dados, capturas, "agir", porta_giz)
        try:
            a2.valor("quadro.alternar", quadro="branco")
            mcp(exe, dados)
        finally:
            a2.fechar()

        print("== a barra abre onde ficou, recolhida ou não")
        a3 = Anotador(exe, dados, capturas, "agir", porta_giz, {"barra": [120, 140], "recolhida": False})
        try:
            h, r = posicao_da_barra()
            confere(h and r[:2] == (120, 140), f"a barra abre na posição gravada ({r[:2]})")
        finally:
            a3.fechar()
        a4 = Anotador(exe, dados, capturas, "agir", porta_giz, {"barra": [120, 140], "recolhida": True})
        try:
            e = a4.estado()
            confere((e["ponta_da_seta"], e["seta_dupla"], e["tipo_de_cota"], e["espessura"], e["cores"][1], e["cor"])
                    == ("cheia", True, "tracos", "grossa", "#123456", "#123456"),
                    f"abre com as escolhas da última vez ({e['ponta_da_seta']}, {e['seta_dupla']}, {e['tipo_de_cota']}, "
                    f"{e['espessura']}, {e['cores'][1]}, {e['cor']})")
            h, r = posicao_da_barra()
            confere(e["barra_recolhida"] and e["modo"] == "mouse livre", "abre recolhida como foi deixada")
            confere(r[2] - r[0] < 100 and r[0] > 120, f"recolhida: botão pequeno à direita da posição da barra ({r})")
        finally:
            a4.fechar()
        a5 = Anotador(exe, dados, capturas, "agir", porta_giz, {"barra": [-30000, 50], "recolhida": False})
        try:
            h, r = posicao_da_barra()
            confere(r[0] > -1000, f"posição de uma tela que não existe mais é ignorada ({r[:2]})")
        finally:
            a5.fechar()

        print("== níveis")
        b = Anotador(exe, dados, capturas, "ler", porta_giz)
        try:
            confere(b.cmd("estado.ler")[0] == 200, "nível 'só ler': estado.ler funciona")
            s, r = b.cmd("texto.escrever", texto="não", x=1, y=1)
            confere(s == 400 and "agir" in r["erro"], "nível 'só ler': desenhar é recusado")
            confere(b.cmd("captura.ler", largura_max=200)[0] == 200, "nível 'só ler': retrato da tela funciona")
        finally:
            b.proc.kill(); b.proc.wait()
        c = Anotador(exe, dados, capturas, "desligado", porta_giz)
        try:
            confere(not (dados / "agente" / "token").exists(), "agentes desligados: o token é apagado")
            confere(c.http("GET", "/api/comandos")[0] == 403, "agentes desligados: a API recusa tudo")
            mcp_desligado(exe, dados)
        finally:
            c.proc.kill(); c.proc.wait()
    finally:
        if giz:
            giz.terminate(); giz.wait()
        shutil.rmtree(tmp, ignore_errors=True)
    print(f"\n{ok_total} ok, {len(falhas)} falha(s)")
    for f in falhas:
        print("  FALHA:", f)
    return 1 if falhas else 0


if __name__ == "__main__":
    sys.exit(main())

# Giz na Tela — © 2026 Heraldo Antunes — Licença MIT (ver LICENSE). Monta a capa do README e a prévia social
"""Capa no modelo "janela clara", o mesmo do Giz Livre: fundo creme liso, ícone, nome e uma linha à esquerda, e a foto
real do anotador numa janela à direita. Sem números nem slogan, para a capa não envelhecer a cada versão.
Desenhada em HTML e fotografada pelo Edge sem janela. Rode depois de foto_capa.py (usa docs/img/capa-foto.png).
Saídas: docs/img/capa.png (1640 x 820, topo do README) e docs/img/previa-social.png (1280 x 640, a mesma página em
escala, enviada à mão em Settings → Social preview do GitHub). Uso: python scripts/fotos/capa.py"""
import base64
import subprocess
import tempfile
from pathlib import Path

RAIZ = Path(__file__).resolve().parents[2]
IMG = RAIZ / "docs" / "img"
EDGE = Path(r"C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe")


def b64(arq):
    return base64.b64encode((IMG / arq).read_bytes()).decode()


# desenhada em 1640 x 820; a barra fica na borda direita da tela, por isso a janela vai inteira, sem sair pela borda
def pagina(largura, altura):
    k = largura / 1640
    px = lambda v: f"{v * k:.1f}px"
    return f"""<!doctype html><html lang="pt-BR"><meta charset="utf-8"><title>Giz na Tela</title><style>
*{{box-sizing:border-box}}html,body{{margin:0;width:{largura}px;height:{altura}px;overflow:hidden}}
body{{position:relative;font-family:'Segoe UI',system-ui,sans-serif;background:#f3f0e8;color:#1b1f1d;-webkit-font-smoothing:antialiased}}
.ic{{position:absolute;left:{px(96)};top:{px(300)};width:{px(84)};height:{px(84)};object-fit:contain}}
h1{{position:absolute;left:{px(96)};top:{px(400)};margin:0;font-size:{px(92)};line-height:1;font-weight:650;letter-spacing:{px(-3.2)};white-space:nowrap}}
p{{position:absolute;left:{px(99)};top:{px(512)};margin:0;font-size:{px(29)};line-height:1.35;color:#4b524e}}
.w{{position:absolute;left:{px(640)};top:{px(140)};width:{px(960)};border-radius:{px(14)};overflow:hidden;background:#fff;
   box-shadow:0 1px 0 #0001,0 {px(18)} {px(40)} {px(-12)} #0000004d,0 0 0 1px #0000001f}}
.w img{{display:block;width:100%}}
</style><img class="ic" src="data:image/png;base64,{b64('icone.png')}" alt=""><h1>Giz na Tela</h1>
<p>Para escrever por cima de<br>qualquer programa, livre e offline.</p>
<div class="w"><img src="data:image/png;base64,{b64('capa-foto.png')}" alt=""></div></html>"""


def fotografar(largura, altura, saida):
    with tempfile.TemporaryDirectory() as tmp:
        arq = Path(tmp) / "capa.html"
        arq.write_text(pagina(largura, altura), encoding="utf-8")
        subprocess.run([str(EDGE), "--headless=new", "--disable-gpu", "--hide-scrollbars", "--force-device-scale-factor=1",
                        f"--user-data-dir={Path(tmp) / 'perfil'}", f"--window-size={largura},{altura}",
                        f"--screenshot={saida}", arq.as_uri()], check=True, capture_output=True, timeout=90)
    print("ok:", saida)


if __name__ == "__main__":
    fotografar(1640, 820, IMG / "capa.png")
    fotografar(1280, 640, IMG / "previa-social.png")

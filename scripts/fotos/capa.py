# Giz na Tela — © 2026 Heraldo Antunes — Licença MIT (ver LICENSE). Monta a capa do README e a prévia social
"""Capa no mesmo estilo da do Giz Livre: fundo verde quadriculado, texto à esquerda e a foto de uso à direita.
Desenhada em HTML e fotografada pelo Edge sem janela. Rode depois de tirar_fotos.py (usa docs/img/uso-*.png).
Saídas: docs/img/capa.png (1600 x 640, topo do README) e docs/img/previa-social.png (1280 x 640, enviada à mão em
Settings → Social preview do GitHub). Uso: python scripts/fotos/capa.py"""
import re
import subprocess
import tempfile
from pathlib import Path

RAIZ = Path(__file__).resolve().parents[2]
IMG = RAIZ / "docs" / "img"
EDGE = Path(r"C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe")


def comandos():
    return len(re.findall(r"\{ nome: '", (RAIZ / "web" / "comandos.js").read_text(encoding="utf-8")))


def ferramentas():
    m = re.search(r"const FERRAMENTAS = \[([^\]]+)\]", (RAIZ / "web" / "comandos.js").read_text(encoding="utf-8"))
    return len(re.findall(r"'", m[1])) // 2


def pagina(largura, social):
    foto_l = 760 if social else 920
    x = largura - foto_l - (40 if social else 56)
    topo = 70 if social else 64
    foto_a = round(foto_l * 1080 / 1920)
    return f"""<!doctype html><html lang="pt-BR"><meta charset="utf-8"><style>
 html,body{{margin:0;width:{largura}px;height:640px;overflow:hidden}}
 body{{position:relative;background:#173d2c;color:#f6f7f0;font-family:'Segoe UI',Arial,sans-serif}}
 .grid{{position:absolute;inset:0;background-image:linear-gradient(#ffffff05 1px,transparent 1px),linear-gradient(90deg,#ffffff05 1px,transparent 1px);background-size:40px 40px}}
 .light{{position:absolute;inset:0;background:radial-gradient(ellipse at 85% 30%,#417e5033,transparent 64%)}}
 .texto{{position:absolute;left:{48 if social else 64}px;top:{56 if social else 62}px;width:{x - 110}px}}
 .eyebrow{{font-size:17px;letter-spacing:2.2px;font-weight:600;color:#c5dfca;margin-bottom:24px}}
 h1{{font-size:{80 if social else 96}px;line-height:1.03;letter-spacing:-3px;margin:0 0 22px;font-weight:700;white-space:nowrap}}
 h1 span{{color:#ffc42b}}
 h2{{font-size:{28 if social else 32}px;line-height:1.28;font-weight:500;margin:0 0 16px;letter-spacing:-.4px}}
 .desc{{font-size:{20 if social else 22}px;line-height:1.4;color:#cee1d3;margin:0}}
 .rule{{height:1px;background:#ffffff26;margin:26px 0 20px}}
 .metricas{{display:flex;gap:{30 if social else 40}px}}
 .metric b{{display:block;font-size:{32 if social else 38}px;line-height:1.1;font-weight:650;letter-spacing:-1px}}
 .metric span{{display:block;font-size:{15 if social else 17}px;margin-top:7px;color:#c5dfca;white-space:nowrap}}
 .foot{{position:absolute;left:{48 if social else 64}px;bottom:30px;font-size:16px;color:#bdd3c3;letter-spacing:.2px}}
 .produto{{position:absolute;left:{x}px;top:{topo}px;width:{foto_l}px;height:{foto_a}px;border:1px solid #ffffff70;border-radius:16px;overflow:hidden;box-shadow:0 25px 70px #071b1490;background:#fff}}
 .produto img{{width:100%;height:100%;display:block}}
 .caption{{position:absolute;left:{x + 8}px;top:{topo + foto_a + 18}px;font-size:15px;color:#c5dfca;letter-spacing:.2px}}
</style><body><div class="grid"></div><div class="light"></div>
<div class="texto">
 <div class="eyebrow">SOFTWARE LIVRE · OFFLINE</div>
 <h1>Giz <span>na Tela</span></h1>
 <h2>Para escrever por cima<br>de qualquer programa.</h2>
 <p class="desc">Slides, PDF, vídeo, navegador.<br>Caneta, setas, holofote e quadro.</p>
 <div class="rule"></div>
 <div class="metricas">
  <div class="metric"><b>{ferramentas()}</b><span>ferramentas</span></div>
  <div class="metric"><b>{comandos()}</b><span>comandos</span></div>
  <div class="metric"><b>3,5 MB</b><span>o programa todo</span></div>
 </div>
</div>
<div class="foot">Grátis · Licença MIT · Windows</div>
<div class="produto"><img src="{(IMG / 'uso-slide.png').as_uri()}" alt=""></div>
<div class="caption">Um slide anotado com o Giz na Tela.</div>
</body></html>"""


def fotografar(html, largura, saida):
    with tempfile.TemporaryDirectory() as tmp:
        arq = Path(tmp) / "capa.html"
        arq.write_text(html, encoding="utf-8")
        subprocess.run([str(EDGE), "--headless=new", "--disable-gpu", "--hide-scrollbars", "--allow-file-access-from-files",
                        f"--user-data-dir={Path(tmp) / 'perfil'}", f"--window-size={largura},640",
                        f"--screenshot={saida}", arq.as_uri()], check=True, capture_output=True, timeout=60)
    print("ok:", saida)


if __name__ == "__main__":
    fotografar(pagina(1600, False), 1600, IMG / "capa.png")
    fotografar(pagina(1280, True), 1280, IMG / "previa-social.png")

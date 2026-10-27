# Giz na Tela — © 2026 Heraldo Antunes — Licença MIT (ver LICENSE). Testes sem abrir o programa (versão e textos)
"""Confere que a versão é a mesma em todos os lugares e que nenhum nome de outro anotador de tela entrou nos arquivos
que vão para a cópia pública. Uso: python tests/testar_versao.py"""
import codecs
import json
import re
import sys
from pathlib import Path

RAIZ = Path(__file__).resolve().parent.parent
falhas = []


def versoes():
    v = {"Cargo.toml": re.search(r'^version = "([^"]+)"', (RAIZ / "Cargo.toml").read_text(encoding="utf-8"), re.M)[1],
         "tauri.conf.json": json.loads((RAIZ / "tauri.conf.json").read_text(encoding="utf-8"))["version"],
         "CITATION.cff": re.search(r"^version: \"?([^\"\n]+)", (RAIZ / "CITATION.cff").read_text(encoding="utf-8"), re.M)[1],
         "CHANGELOG.md": re.search(r"^## \[?(\d+\.\d+\.\d+)", (RAIZ / "CHANGELOG.md").read_text(encoding="utf-8"), re.M)[1]}
    iss = RAIZ / "installer" / "giz-na-tela.iss"
    if iss.exists():
        v["giz-na-tela.iss"] = re.search(r'#define Versao "([^"]+)"', iss.read_text(encoding="utf-8-sig"))[1]
    return v


v = versoes()
if len(set(v.values())) != 1:
    falhas.append(f"versões diferentes: {v}")
else:
    print("  ok   versão igual em todos os lugares:", next(iter(v.values())), f"({', '.join(v)})")

# créditos de ideias ficam neutros: nenhum nome de outro anotador nos arquivos públicos (código, páginas, README)
# (cifrados em rot13 para que nem este teste leve os nomes à cópia pública)
PROIBIDOS = re.compile(codecs.decode(r"cbvagbsvk|tvax|ccvax|rcvp ?cra|mbbzvg", "rot13"), re.I)
publicos = [p for pat in ("src/*.rs", "web/*", "README.md", "CHANGELOG.md", "installer/*", "tests/*.py", "scripts/*")
            for p in RAIZ.glob(pat) if p.is_file() and p.suffix in (".rs", ".html", ".js", ".md", ".iss", ".txt", ".py", ".ps1")]
for p in publicos:
    achado = PROIBIDOS.search(p.read_text(encoding="utf-8-sig", errors="replace"))
    if achado:
        falhas.append(f"nome de outro programa em {p.relative_to(RAIZ)}: {achado[0]}")
print(f"  {'ok  ' if not any('nome de outro' in f for f in falhas) else 'FALHA'} nenhum nome de outro anotador em "
      f"{len(publicos)} arquivos públicos")

# cabeçalho de uma linha com a licença em todo arquivo próprio
for p in publicos:
    if p.suffix in (".rs", ".html", ".js", ".py", ".iss", ".ps1"):
        topo = p.read_text(encoding="utf-8-sig").splitlines()[:2]
        if not any("Giz na Tela" in t and "MIT" in t for t in topo):
            falhas.append(f"sem cabeçalho de licença: {p.relative_to(RAIZ)}")
print(f"  {'ok  ' if not any('cabeçalho' in f for f in falhas) else 'FALHA'} cabeçalho de licença nos arquivos próprios")

# a CSP do programa (tauri.conf.json) bloqueia style="" escrito nos elementos: a cor e o tamanho somem sem erro nenhum
for p in (RAIZ / "web").glob("*.html"):
    for n, linha in enumerate(p.read_text(encoding="utf-8").splitlines(), 1):
        if re.search(r"<[a-z][^>]*\sstyle=\"", linha):
            falhas.append(f"style=\"\" no HTML (a CSP bloqueia): {p.name}:{n}")
print(f"  {'ok  ' if not any('style=' in f for f in falhas) else 'FALHA'} nenhum style=\"\" nas páginas (a CSP bloquearia)")

for f in falhas:
    print("  FALHA:", f)
sys.exit(1 if falhas else 0)

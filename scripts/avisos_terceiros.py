# Giz na Tela — © 2026 Heraldo Antunes — Licença MIT (ver LICENSE). Gera THIRD-PARTY-NOTICES.md das bibliotecas Rust
"""Lista as bibliotecas que entram no GizNaTela.exe (dependências normais, no Windows), com versão, licença e o texto
da licença tirado do próprio pacote baixado pelo cargo. MIT e Apache pedem que o aviso acompanhe o programa: o
arquivo vai no instalador e na versão portátil. Uso (PowerShell, com o cargo no PATH): python scripts/avisos_terceiros.py"""
import json
import subprocess
from pathlib import Path

RAIZ = Path(__file__).resolve().parent.parent
meta = json.loads(subprocess.run(["cargo", "metadata", "--format-version", "1", "--filter-platform",
                                  "x86_64-pc-windows-msvc"], cwd=RAIZ, capture_output=True, check=True).stdout)
pacotes = {p["id"]: p for p in meta["packages"]}
nos = {n["id"]: n for n in meta["resolve"]["nodes"]}
raiz = meta["resolve"]["root"]

# só o que vai para o programa: dependências normais (nem de build nem de teste), a partir do giz-na-tela
vistos, fila = set(), [raiz]
while fila:
    i = fila.pop()
    for d in nos[i]["deps"]:
        if any(k["kind"] is None for k in d["dep_kinds"]) and d["pkg"] not in vistos:
            vistos.add(d["pkg"])
            fila.append(d["pkg"])

lista = sorted((pacotes[i] for i in vistos), key=lambda p: (p["name"], p["version"]))
partes = ["# Avisos de terceiros do Giz na Tela\n",
          "O Giz na Tela (licença MIT, ver LICENSE) é feito com as bibliotecas abaixo, cada uma com a sua licença. "
          "Usa também o WebView2 do Windows, que já vem com o sistema e não é distribuído com o programa.\n",
          "| Biblioteca | Versão | Licença |", "|---|---|---|"]
partes += [f"| {p['name']} | {p['version']} | {p.get('license') or 'ver texto'} |" for p in lista]
partes.append("\n## Textos das licenças\n")
for p in lista:
    pasta = Path(p["manifest_path"]).parent
    arqs = sorted(a for a in pasta.iterdir() if a.is_file() and a.name.upper().startswith(("LICENSE", "LICENCE", "COPYING", "NOTICE")))
    partes.append(f"### {p['name']} {p['version']}\n")
    if not arqs:
        partes.append(f"Licença: {p.get('license')}. O pacote não traz o texto; vale o texto padrão dessa licença.\n")
    for a in arqs:
        partes.append(f"`{a.name}`\n\n```\n{a.read_text(encoding='utf-8', errors='replace').strip()}\n```\n")
(RAIZ / "THIRD-PARTY-NOTICES.md").write_text("\n".join(partes), encoding="utf-8", newline="\n")
print(f"ok: {len(lista)} bibliotecas em THIRD-PARTY-NOTICES.md")

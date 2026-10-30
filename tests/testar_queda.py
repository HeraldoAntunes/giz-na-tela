# Giz na Tela — © 2026 Heraldo Antunes — Licença MIT (ver LICENSE). Reabrir logo depois de uma queda, 10 vezes
"""Depois de uma queda (o exe morto à força), o processo principal do WebView2 continua vivo segurando o perfil, e a
reabertura falhava ao criar a janela. Este teste mata o GizNaTela.exe à força e o reabre logo em seguida, com a mesma
pasta de dados e o mesmo perfil do WebView2 (de rascunho), 10 vezes: tem de abrir nas 10, sem parada inesperada
(erros.txt) e sem cair na mensagem de "não conseguiu abrir a janela" (o programa sairia).
Uso: cargo build --release; python tests/testar_queda.py [exe]"""
import os
import subprocess
import sys
import tempfile
import time
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from testar_comandos import PASTAS, PERFIS, fechar_webview_do_perfil, porta_livre   # noqa: E402

EXE = Path(sys.argv[1]) if len(sys.argv) > 1 else Path(__file__).resolve().parent.parent / "target" / "release" / "GizNaTela.exe"
VEZES = 10


def principal():
    print(f"exe: {EXE}")
    dados = Path(tempfile.mkdtemp())
    PASTAS.append(dados)
    perfil = tempfile.mkdtemp(prefix="giz-perfil-")
    PERFIS.append(perfil)
    env = {**os.environ, "WEBVIEW2_USER_DATA_FOLDER": perfil}
    abriu = 0
    try:
        for i in range(VEZES + 1):   # a primeira abertura é a normal; as 10 seguintes vêm depois de uma queda
            (dados / "agente" / "porta").unlink(missing_ok=True)
            p = subprocess.Popen([str(EXE), "--dados", str(dados), "--porta-api", str(porta_livre())], env=env,
                                 stderr=subprocess.DEVNULL)
            limite = time.time() + 30
            while time.time() < limite and p.poll() is None and not (dados / "agente" / "porta").exists():
                time.sleep(0.2)
            time.sleep(1.5)
            vivo = p.poll() is None and (dados / "agente" / "porta").exists()
            if i:
                abriu += vivo
                print(f"  reabertura {i} depois da queda: {'abriu' if vivo else 'NÃO ABRIU'}")
            elif not vivo:
                print("  a primeira abertura falhou: o teste não vale")
                return 1
            if p.poll() is None:
                p.kill()   # a queda: só o exe, o WebView2 dele fica
            p.wait()
    finally:
        fechar_webview_do_perfil(perfil)
    erros = (dados / "erros.txt").exists()
    log = (dados / "janelas.log").read_text(encoding="utf-8") if (dados / "janelas.log").exists() else ""
    print(f"  janelas.log: {log.count('sobrou de uma queda')} fechamento(s) de WebView2 órfão anotado(s)")
    ok = abriu == VEZES and not erros
    if not ok:   # a falha é rara (1 em uns 100 em 29/10/2026): mostra o que ficou antes de as pastas serem apagadas
        print("  erros.txt:", (dados / "erros.txt").read_text(encoding="utf-8") if erros else "(não há)")
        print("  janelas.log:", log or "(vazio)")
    print(f"\n{'ok' if ok else 'FALHA'}: abriu {abriu} de {VEZES} depois da queda, erros.txt={erros}")
    return 0 if ok else 1


if __name__ == "__main__":
    sys.exit(principal())

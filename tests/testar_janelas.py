# Giz na Tela — © 2026 Heraldo Antunes — Licença MIT (ver LICENSE). Falha do WebView2 ao criar as janelas
"""O WebView2 às vezes falha ao criar a janela (0x80070578), por exemplo com outro programa abrindo e fechando janelas
dele ao mesmo tempo. A compilação com debug-assertions faz as primeiras tentativas falharem de propósito com
GIZ_TESTE_FALHAR_JANELA=n. Uma falha: o programa tem de abrir na tentativa seguinte. Falhas em todas: tem de sair em
ordem (sem parada inesperada nem erros.txt), com o detalhe só no janelas.log.
Tem de ser a compilação otimizada: a de debug (target/debug), sem otimização, já erra a primeira criação de verdade
quase sempre (0x80070578 na camada, visto em 29/10/2026), e a tentativa seguinte à falha forçada também cai.
Uso: cargo build --release --config profile.release.debug-assertions=true --target-dir target/teste-otimizado
     python tests/testar_janelas.py [exe]"""
import os
import socket
import subprocess
import sys
import tempfile
import time
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from testar_comandos import PASTAS, PERFIS, fechar_webview_do_perfil   # noqa: E402  (fecha o WebView2 órfão e apaga as pastas no fim)

EXE = Path(sys.argv[1]) if len(sys.argv) > 1 else Path(__file__).resolve().parent.parent / "target" / "teste-otimizado" / "release" / "GizNaTela.exe"
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


def abrir(falhar):
    dados = Path(tempfile.mkdtemp())
    PASTAS.append(dados)
    perfil = tempfile.mkdtemp(prefix="giz-perfil-")
    PERFIS.append(perfil)
    env = {**os.environ, "GIZ_TESTE_FALHAR_JANELA": str(falhar), "WEBVIEW2_USER_DATA_FOLDER": perfil}
    p = subprocess.Popen([str(EXE), "--dados", str(dados), "--porta-api", str(porta_livre())], env=env,
                         stderr=subprocess.DEVNULL)
    limite = time.time() + 30
    while time.time() < limite and p.poll() is None and not (dados / "agente" / "porta").exists():
        time.sleep(0.2)
    time.sleep(1)
    return p, dados


def linhas(arq):
    return arq.read_text(encoding="utf-8").splitlines() if arq.exists() else []


def principal():
    print(f"exe: {EXE}")
    p, dados = abrir(1)
    vivo = p.poll() is None
    confere(vivo and (dados / "agente" / "porta").exists() and not (dados / "erros.txt").exists(),
            f"uma falha do WebView2: abre na tentativa seguinte (vivo={vivo}, erros.txt={(dados / 'erros.txt').exists()})")
    confere(len(linhas(dados / "janelas.log")) == 1, f"a falha fica anotada no janelas.log ({len(linhas(dados / 'janelas.log'))} linha)")
    if vivo:
        p.kill()
    p.wait()
    time.sleep(1)
    fechar_webview_do_perfil(PERFIS[-1])

    p, dados = abrir(9)
    try:
        codigo = p.wait(30)
    except subprocess.TimeoutExpired:
        p.kill()
        codigo = "não saiu"
    confere(codigo == 1 and not (dados / "erros.txt").exists(),
            f"falha em todas: sai em ordem, sem parada inesperada (código {codigo}, erros.txt={(dados / 'erros.txt').exists()})")
    confere(len(linhas(dados / "janelas.log")) == 3, f"três tentativas anotadas ({len(linhas(dados / 'janelas.log'))} linhas)")

    print(f"\n{ok_total} ok, {len(falhas)} falha(s)")
    sys.exit(1 if falhas else 0)


if __name__ == "__main__":
    principal()

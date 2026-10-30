# Giz na Tela — © 2026 Heraldo Antunes — Licença MIT (ver LICENSE). Migração da 1.3.x sem perder o desenho aberto
"""O desinstalador da 1.3.x (Inno Setup) fecha à força todo GizNaTela.exe da conta. Este teste deixa uma cópia
portátil aberta com desenho, instala a 1.3.2 publicada e roda o instalador novo em silêncio: a cópia aberta não pode
morrer sem perguntar. Depois fecha a cópia (como o professor, confirmando) e confere que a instalação termina. Também
confere que uma cópia aberta sem desenho é fechada pelo caminho normal e a instalação segue sozinha.
MEXE NA INSTALAÇÃO DA CONTA e abre o programa na tela; no fim, devolve a 1.3.2 publicada.
Uso: python tests/testar_migracao.py <instalador novo> <instalador da 1.3.2 publicada>"""
import shutil
import subprocess
import sys
import tempfile
import time
import winreg
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import testar_comandos as T   # noqa: E402

RAIZ = r"Software\Microsoft\Windows\CurrentVersion\Uninstall"
INNO = RAIZ + r"\{4BB282B2-1BF2-444F-BCB1-0F6E51B46035}_is1"
NOVA = RAIZ + r"\GizNaTela"


def chave(sub):
    try:
        with winreg.OpenKey(winreg.HKEY_CURRENT_USER, sub) as k:
            return winreg.QueryValueEx(k, "DisplayVersion")[0]
    except OSError:
        return None


def instalar_inno(inno):
    subprocess.run([inno, "/VERYSILENT", "/SUPPRESSMSGBOXES", "/NORESTART", "/CURRENTUSER"], check=True)
    T.confere(chave(INNO) == "1.3.2", "a 1.3.2 do Inno está instalada")


def desinstalar_nova():
    try:
        with winreg.OpenKey(winreg.HKEY_CURRENT_USER, NOVA) as k:
            cmd = winreg.QueryValueEx(k, "QuietUninstallString")[0]
    except OSError:
        return
    alvo = cmd.split('"')[1]
    subprocess.run([alvo, "--desinstalar", "/S"])
    for _ in range(60):
        if chave(NOVA) is None:
            break
        time.sleep(0.5)


def caso(instalador, inno, com_desenho):
    pasta = Path(tempfile.mkdtemp(prefix="giz-portatil-"))
    exe = pasta / "GizNaTela.exe"
    shutil.copy(instalador, exe)   # o mesmo binário, com o nome do programa: abre normal, como um portátil
    a = T.Anotador(exe, Path(tempfile.mkdtemp()), Path(tempfile.mkdtemp()), "agir")
    try:
        if com_desenho:
            a.valor("texto.escrever", texto="desenho que não pode sumir", x=300, y=300)
        antes = a.estado()["desenhos"]
        proc = subprocess.Popen([str(instalador), "/S"])
        if com_desenho:
            time.sleep(20)
            vivo = a.proc.poll() is None
            desenhos = a.estado()["desenhos"] if vivo else None
            T.confere(vivo and desenhos == antes and proc.poll() is None,
                      f"cópia aberta com desenho: continua viva com o desenho e a instalação espera (vivo={vivo}, "
                      f"desenhos={desenhos} de {antes}, instalador terminou={proc.poll() is not None})")
            if vivo:
                a.fechar()   # o professor confirma o fechar (segundo toque no X)
        try:
            codigo = proc.wait(60)
        except subprocess.TimeoutExpired:
            proc.kill()
            codigo = "não terminou"
        if not com_desenho:
            T.confere(a.proc.poll() is not None, "cópia aberta sem desenho: fechou pelo caminho normal")
        T.confere(codigo == 0 and chave(INNO) is None and chave(NOVA) is not None,
                  f"instalação terminou depois do fechar (código {codigo}, Inno={chave(INNO)}, nova={chave(NOVA)})")
    finally:
        if a.proc.poll() is None:
            a.fechar()
        shutil.rmtree(pasta, ignore_errors=True)


def principal():
    if len(sys.argv) != 3:
        sys.exit(__doc__)
    instalador, inno = Path(sys.argv[1]).resolve(), Path(sys.argv[2]).resolve()
    try:
        for com_desenho in (True, False):
            desinstalar_nova()
            instalar_inno(inno)
            caso(instalador, inno, com_desenho)
    finally:
        desinstalar_nova()
        if chave(INNO) is None:
            instalar_inno(inno)   # devolve a versão publicada
    print(f"\n{T.ok_total} ok, {len(T.falhas)} falha(s)")
    sys.exit(1 if T.falhas else 0)


if __name__ == "__main__":
    principal()

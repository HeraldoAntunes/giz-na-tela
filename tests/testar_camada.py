# Giz na Tela — © 2026 Heraldo Antunes — Licença MIT (ver LICENSE). Casos da auditoria de 29/10/2026 na camada
"""Roda os casos da auditoria independente (docs/AUDITORIA-2026-10-29-RESPOSTA.md) no WebView2 de verdade: abre a
compilação de teste (com debug-assertions, que tem a rota /api/teste/camada) com pastas temporárias e manda eventos
de ponteiro sintéticos à camada. Eventos sintéticos não são caneta física: pressão, captura e vários monitores reais
ficam para a mão do professor.
Uso (PowerShell, com o cargo no PATH):
  cargo build --release --config profile.release.debug-assertions=true --target-dir target/teste-otimizado
  python tests/testar_camada.py"""
import json
import sys
import tempfile
import time
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import testar_comandos as T   # noqa: E402

RAIZ = Path(__file__).resolve().parent.parent
# a compilação otimizada com debug-assertions: a de debug, sem otimização, às vezes erra as janelas ao abrir (ver
# tests/testar_janelas.py) e o teste caía sem a API
EXE = RAIZ / "target" / "teste-otimizado" / "release" / "GizNaTela.exe"

# ponteiro sintético: mouse (id 1) por padrão; traco() desenha de um ponto a outro em 6 passos
PRELUDIO = r"""
const ev = (tipo, x, y, o = {}) => cv.dispatchEvent(new PointerEvent(tipo, { bubbles: true, cancelable: true,
  pointerId: 1, pointerType: 'mouse', isPrimary: true, clientX: x, clientY: y,
  button: tipo === 'pointermove' ? -1 : 0, buttons: tipo === 'pointerup' ? 0 : 1,
  pressure: tipo === 'pointerup' ? 0 : 0.5, ...o }));
const traco = (x0, y0, x1, y1, o = {}) => {
  ev('pointerdown', x0, y0, o);
  for (let i = 1; i <= 6; i++) ev('pointermove', x0 + (x1 - x0) * i / 6, y0 + (y1 - y0) * i / 6, o);
  ev('pointerup', x1, y1, o);
};
const espera = ms => new Promise(r => setTimeout(r, ms));
const zerar = () => { terminarEscrita(true); S.tracos = []; S.passado = []; S.futuro = []; S.some = false;
  S.ferramenta = 'caneta'; S.largura = 4; sel = new Set(); redesenhar(); avisarBarra(); };
zerar();
"""


def js(a, corpo):
    s, r = a.http("POST", "/api/teste/camada", cru=(PRELUDIO + corpo).encode())
    if s != 200:
        raise AssertionError(f"teste/camada: {s} {r}")
    return r["valor"]


def novo():
    return T.Anotador(EXE, Path(tempfile.mkdtemp()), Path(tempfile.mkdtemp()), "agir")


def fechar_recusado(a, como):
    s, r = a.cmd("programa.fechar")
    time.sleep(1.5)
    vivo = a.proc.poll() is None
    T.confere(s == 400 and vivo, f"{como}: o agente não fecha e a tinta fica (resposta {s}, vivo={vivo}, {r.get('erro', r)})")


def principal():
    if not EXE.exists():
        sys.exit(f"falta {EXE}: rode antes a compilação de teste (ver o começo deste arquivo)")

    a = novo()
    try:
        # Mão: um toque sem arrastar não pode apagar o Refazer
        v = js(a, """traco(200, 200, 400, 200); traco(200, 300, 400, 300); camada.desfazer();
          const antes = S.futuro.length; S.ferramenta = 'mover';
          ev('pointerdown', 300, 200); ev('pointerup', 300, 200);
          return { antes, depois: S.futuro.length, tracos: S.tracos.length };""")
        T.confere(v["antes"] == 1 and v["depois"] == 1, f"Mão: toque sem arrastar mantém o Refazer ({v})")

        # toque único de caneta: o ponto aparece na tela
        v = js(a, """ev('pointerdown', 300, 300); ev('pointerup', 300, 300);
          const r = devicePixelRatio || 1;
          return { tracos: S.tracos.length, alfa: ctx.getImageData(Math.round(300 * r), Math.round(300 * r), 1, 1).data[3] };""")
        T.confere(v["tracos"] == 1 and v["alfa"] > 0, f"toque único desenha o ponto ({v})")

        # segundo toque (outro pointerId) não troca o traço nem cria uma diagonal entre os dois
        v = js(a, """const pen = id => ({ pointerId: id, pointerType: 'pen', isPrimary: id === 11 });
          ev('pointerdown', 100, 100, pen(11)); ev('pointermove', 150, 100, pen(11));
          ev('pointerdown', 500, 500, pen(12)); ev('pointermove', 520, 520, pen(12));
          ev('pointermove', 200, 100, pen(11)); ev('pointerup', 520, 520, pen(12)); ev('pointerup', 200, 100, pen(11));
          let salto = 0, chegou = false;
          for (const t of S.tracos) for (let i = 1; i < (t.pts || []).length; i++) {
            salto = Math.max(salto, Math.hypot(t.pts[i].x - t.pts[i - 1].x, t.pts[i].y - t.pts[i - 1].y));
            if (t.pts[i].x === 200 && t.pts[i].y === 100) chegou = true;
          }
          return { tracos: S.tracos.length, salto, chegou };""")
        T.confere(v["salto"] < 100 and v["chegou"], f"segundo toque não cria diagonal e o primeiro traço segue ({v})")

        # pressão zero é uma pressão de verdade (traço fino), não "sem pressão"
        v = js(a, """return { zero: largura({ p: 0 }, { largura: 4 }), sem: largura({}, { largura: 4 }) };""")
        T.confere(abs(v["zero"] - 1.4) < 1e-9 and abs(v["sem"] - 4) < 1e-9, f"pressão zero dá o traço fino ({v})")

        # tinta que some, deixada noutra tela: ao voltar, não conta nem pede confirmação ao fechar
        v = js(a, """const orig = S.chaveTela; camada.tela('auditoriaA'); S.some = true; traco(200, 400, 400, 400);
          camada.tela('auditoriaB'); await espera(3800);
          const fora = estado(); camada.tela('auditoriaA'); const volta = estado();
          S.some = false; camada.tela('auditoriaB'); camada.tela(orig); avisarBarra();
          return { conteudoFora: fora.conteudo ?? null, tracosAoVoltar: volta.tracos };""")
        T.confere(v["tracosAoVoltar"] == 0 and (v["conteudoFora"] or 0) == 0, f"tinta que some expirada noutra tela não conta ({v})")

        # casos do harness da auditoria que a suíte ainda não tinha (regressão)
        # cada tela tem o próprio Desfazer: desfazer numa não mexe na outra (telas novas: a auditoriaA guarda um passo do
        # caso da tinta que some)
        v = js(a, """const orig = S.chaveTela; camada.tela('historicoA'); traco(200, 200, 400, 200);
          camada.tela('historicoB'); traco(200, 300, 400, 300); camada.desfazer();
          const b = S.tracos.length; camada.tela('historicoA'); const aa = S.tracos.length, desfazerA = S.passado.length;
          camada.tela('historicoB'); camada.tela(orig); avisarBarra();
          return { b, a: aa, desfazerA };""")
        T.confere(v["b"] == 0 and v["a"] == 1 and v["desfazerA"] == 1, f"histórico independente por tela ({v})")
        # limite de pontos de um traço pela API: 5000 entram, 5001 são recusados inteiros
        js(a, "return true;")   # zera a tela
        s1, _ = a.cmd("traco.desenhar", pontos=[[100 + i % 600, 100 + (i // 600) * 5] for i in range(5000)])
        s2, r2 = a.cmd("traco.desenhar", pontos=[[100 + i % 600, 100 + (i // 600) * 5] for i in range(5001)])
        n = a.estado()["desenhos"]
        T.confere(s1 == 200 and s2 == 400 and n == 1, f"traço com 5000 pontos entra e com 5001 é recusado sem nada pela metade ({s1}, {s2}, desenhos={n})")
    finally:
        a.fechar()

    # fechar com o desenho noutra tela (a tela de agora vazia)
    a = novo()
    try:
        js(a, """const orig = S.chaveTela; camada.tela('auditoriaA'); traco(200, 200, 400, 200);
          camada.tela('auditoriaB'); avisarBarra(); return true;""")
        fechar_recusado(a, "desenho noutra tela, a de agora vazia")
    finally:
        a.fechar()

    # fechar com um texto ainda em edição (nada na lista de desenhos)
    a = novo()
    try:
        v = js(a, """S.ferramenta = 'texto'; ev('pointerdown', 300, 300); await espera(100);
          escrita.el.innerText = 'texto em edição'; escrita.el.dispatchEvent(new Event('input')); avisarBarra();
          return { tracos: S.tracos.length, escrevendo: !!escrita };""")
        fechar_recusado(a, f"texto em edição ({v})")
    finally:
        a.fechar()

    # estado vazio ou null (a camada sem responder, o estado sem a contagem): na dúvida há desenho, e não fecha
    # (lacuna N-C1 da auditoria do modelo nativo, 29/10/2026)
    a = novo()
    try:
        js(a, """traco(200, 200, 400, 200); camada.estado = () => null;
          window.ponte.estadoCamada({ tracos: null, conteudo: null }); return true;""")
        fechar_recusado(a, "estado da camada null, com desenho")
    finally:
        a.fechar()

    # a rota de teste só existe na compilação de teste: o programa de verdade (release) responde 404
    rel = RAIZ / "target" / "release" / "GizNaTela.exe"
    if rel.exists():
        a = T.Anotador(rel, Path(tempfile.mkdtemp()), Path(tempfile.mkdtemp()), "agir")
        try:
            s, _ = a.http("POST", "/api/teste/camada", cru=b"return 1")
            T.confere(s == 404, f"a compilação de verdade não tem a rota /api/teste/camada (resposta {s})")
        finally:
            a.fechar()

    print(f"\n{T.ok_total} ok, {len(T.falhas)} falha(s)")
    sys.exit(1 if T.falhas else 0)


if __name__ == "__main__":
    principal()

// Giz na Tela — © 2026 Heraldo Antunes — Licença MIT (ver LICENSE). Delete que limpa a tela, sem janela
// Roda o script da barra e o da camada no Node, com um DOM de mentira e a ponte do host simulada, e confere a
// confirmação do Delete: manter a tecla pressionada (repetição do teclado) não vale pelo segundo toque, e o Delete
// armado numa tela não limpa a outra depois de trocar de tela (achados 1 e 2 da revisão do Codex, Ordem 23). Também:
// o Espaço não é atalho (vídeo por baixo), a I escolhe a mão, e o tempo da tinta que some (2, 3, 5 e 10 s).
// Uso: node tests/testar_teclado.mjs
import { readFileSync } from 'node:fs';
import vm from 'node:vm';

const WEB = new URL('../web/', import.meta.url);
const scriptDe = (arquivo) => {
  const html = readFileSync(new URL(arquivo, WEB), 'utf8');
  return [...html.matchAll(/<script>([\s\S]*?)<\/script>/g)].map(m => m[1]).join('\n');
};

// elemento de mentira: aceita qualquer propriedade e qualquer chamada (o que a página desenha não importa aqui)
const coisa = () => new Proxy(function () {}, {
  get: (_, k) => (k === Symbol.toPrimitive ? () => 0 : k === 'then' ? undefined : coisa()),
  set: () => true,
  apply: () => coisa(),
});

function pagina(arquivo, ponte, extras = '') {
  const teclas = [];
  const document = new Proxy({}, {
    get: (_, k) => {
      if (k === 'getElementById') return () => coisa();
      if (k === 'querySelectorAll') return () => [];
      if (k === 'addEventListener') return (tipo, f) => { if (tipo === 'keydown') teclas.push(f); };
      return coisa();
    },
  });
  // a página enxerga o window como o próprio global, igual no navegador
  const window = { ponte, CONFIG: {}, document, console, Date, performance, addEventListener() {}, devicePixelRatio: 1,
    innerWidth: 800, innerHeight: 600, setTimeout: () => 0, clearTimeout() {}, setInterval: () => 0, clearInterval() {},
    requestAnimationFrame: () => 0, matchMedia: () => coisa(), PointerEvent: class {}, Image: class {} };
  window.window = window;
  const ctx = vm.createContext(window);
  vm.runInContext(readFileSync(new URL('comandos.js', WEB), 'utf8'), ctx);
  vm.runInContext(scriptDe(arquivo) + extras, ctx);
  let barrados = 0;
  return { window, rodar: c => vm.runInContext(c, ctx), barrados: () => barrados, apertar: (key, repeat = false) => teclas.forEach(f => f({ key, repeat,
    altKey: false, ctrlKey: false, preventDefault() { barrados++; } })) };
}

const espera = () => new Promise(r => setImmediate(r));
let falhas = 0;
const confere = (ok, msg) => { console.log(`  ${ok ? 'ok  ' : 'FALHA:'} ${msg}`); if (!ok) falhas++; };

// a barra com duas telas, cada uma com a sua tinta; a ponte anota cada "limpar" e em que tela foi
function barra() {
  const tinta = { A: 3, B: 2 }, limpou = [], ferramentas = [], fixados = [], tempos = [], guardadas = [];
  let atual = 'A';
  const telas = () => ({ telas: [{ nome: 'A' }, { nome: 'B' }], atual: atual === 'A' ? 1 : 2 });
  const ponte = {
    camada: async (nome, valor) => {
      if (nome === 'tela') atual = valor;
      if (nome === 'limpar') { limpou.push(atual); tinta[atual] = 0; }
      if (nome === 'ferramenta') ferramentas.push(valor);
      if (nome === 'someTempo') tempos.push(valor);
      return { tracos: tinta[atual] };
    },
    telas: async () => telas(),
    mudarTela: async (n) => { atual = n === 1 ? 'A' : 'B'; return telas(); },
    tecla() {}, fixar: async v => { fixados.push(v); return v; }, recolher: async () => {}, preferencias: async p => { guardadas.push(p); },
  };
  const p = pagina('barra.html', ponte);
  return { ...p, limpou, ferramentas, fixados, tempos, guardadas };
}

console.log('== barra: Delete mantido pressionado');
{
  const b = barra();
  await espera(); await espera();
  b.apertar('Delete'); await espera();
  for (let i = 0; i < 5; i++) { b.apertar('Delete', true); await espera(); }
  confere(b.limpou.length === 0, `a repetição do teclado não confirma a limpeza (limpou: ${b.limpou.length})`);
  b.apertar('Delete'); await espera();
  confere(b.limpou.length === 1, 'soltar e apertar de novo limpa (o segundo toque de verdade)');
}

console.log('== barra: Delete armado numa tela e a troca de tela');
{
  const b = barra();
  await espera(); await espera();
  b.apertar('Delete'); await espera();
  b.apertar('y'); await espera(); await espera();
  b.apertar('Delete'); await espera();
  confere(!b.limpou.includes('B'), `o Delete armado na tela A não limpa a tela B (limpou: ${b.limpou.join(',') || 'nada'})`);
  b.apertar('Delete'); await espera();
  confere(b.limpou.join(',') === 'B', 'na tela B, dois toques limpam a tela B');
}

console.log('== barra: o Espaço não é atalho (pausar o vídeo de baixo) e a I escolhe a mão');
{
  const b = barra();
  await espera(); await espera();
  await b.window.executar('tela.fixar', { ligado: true }, 'botao');
  const [nf, nx] = [b.ferramentas.length, b.fixados.length];
  const antes = b.barrados();
  b.apertar(' '); await espera(); await espera();
  confere(b.ferramentas.length === nf, `o Espaço não troca a ferramenta (trocas: ${b.ferramentas.slice(nf).join(',') || 'nenhuma'})`);
  confere(b.fixados.length === nx, `o Espaço não tira do Mouse livre (fixar: ${b.fixados.slice(nx).join(',') || 'nada'})`);
  confere(b.barrados() === antes + 1, 'na barra, o Espaço não aperta o botão que tem o foco');
  b.window.tecla(' '); await espera(); await espera();
  confere(b.ferramentas.length === nf, 'o Espaço que vem da camada também não faz nada');
  confere(!Object.hasOwn(b.window.GizTela.ATALHOS, ' '), 'o Espaço não está no mapa de atalhos');
  b.apertar('i'); await espera(); await espera();
  confere(b.ferramentas.at(-1) === 'mover', `a I escolhe a mão (ferramenta: ${b.ferramentas.at(-1)})`);
  confere(b.fixados.at(-1) === false, 'escolher a mão pela I volta a escrever, como as outras ferramentas');
}

console.log('== barra: tempo da tinta que some (2, 3, 5 e 10 s), guardado nos ajustes');
{
  const b = barra();
  await espera(); await espera();
  confere(b.window.estadoPublico().tinta_some_segundos === 3, 'de fábrica, 3 segundos');
  for (const s of [2, 3, 5, 10]) {
    await b.window.executar('tinta.tempo', { segundos: s }, 'botao');
    confere(b.tempos.at(-1) === s && b.window.estadoPublico().tinta_some_segundos === s, `${s} s chega à camada e ao estado`);
    confere(b.guardadas.at(-1)?.someSegundos === s, `${s} s fica guardado nos ajustes`);
  }
  for (const ruim of [4, 0, 2.5, '3']) {
    let recusou = false;
    try { await b.window.executar('tinta.tempo', { segundos: ruim }, 'agente'); } catch { recusou = true; }
    confere(recusou, `tempo ${JSON.stringify(ruim)} é recusado`);
  }
  await b.window.aplicarPreferencias({ someSegundos: 5 });
  confere(b.window.estadoPublico().tinta_some_segundos === 5, 'abre com o tempo guardado da última vez (5 s)');
  await b.window.aplicarPreferencias({ someSegundos: 7 });
  confere(b.window.estadoPublico().tinta_some_segundos === 5, 'tempo inválido mexido à mão no arquivo não entra');
}

console.log('== camada: o traço da tinta que some dura o tempo escolhido');
{
  const ponte = new Proxy({}, { get: (o, k) => o[k] ?? (() => {}) });
  let c;
  try { c = pagina('camada.html', ponte); } catch (e) { confere(false, `a camada não rodou no Node: ${e.message}`); }
  if (c) {
    c.window.camada.some(true);
    for (const s of [2, 3, 5, 10]) {
      c.window.camada.someTempo(s);
      c.rodar("acrescentar({ pts: [[10, 10, 0.5], [20, 20, 0.5]], cor: '#2e9d4a', largura: 4 })");
      const resta = c.rodar('S.tracos.at(-1).ate - performance.now()');
      confere(Math.abs(resta - s * 1000) < 100, `${s} s: o traço some em ${(resta / 1000).toFixed(2)} s`);
    }
    c.window.camada.someTempo(7);
    c.rodar("acrescentar({ pts: [[10, 10, 0.5], [20, 20, 0.5]], cor: '#2e9d4a', largura: 4 })");
    const resta = c.rodar('S.tracos.at(-1).ate - performance.now()');
    confere(Math.abs(resta - 10000) < 100, 'tempo fora da lista (7) não muda o tempo de agora (10 s)');
  }
}

console.log('== camada: a repetição do Delete não chega à barra');
{
  const mandadas = [];
  const ponte = new Proxy({ tecla: k => mandadas.push(k) }, { get: (o, k) => o[k] ?? (() => {}) });
  let c;
  try { c = pagina('camada.html', ponte); } catch (e) { confere(false, `a camada não rodou no Node: ${e.message}`); }
  if (c) {
    c.apertar('Delete'); c.apertar('Delete', true); c.apertar('Delete', true);
    confere(mandadas.length === 1, `só o primeiro Delete vai à barra (foram ${mandadas.length})`);
  }
}

console.log(falhas ? `\n${falhas} FALHA(S)` : '\ntudo ok');
process.exit(falhas ? 1 : 0);

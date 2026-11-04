// Giz na Tela — © 2026 Heraldo Antunes — Licença MIT (ver LICENSE). Delete que limpa a tela, sem janela
// Roda o script da barra e o da camada no Node, com um DOM de mentira e a ponte do host simulada, e confere a
// confirmação do Delete: manter a tecla pressionada (repetição do teclado) não vale pelo segundo toque, e o Delete
// armado numa tela não limpa a outra depois de trocar de tela (achados 1 e 2 da revisão do Codex, Ordem 23).
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
  return { window, apertar: (key, repeat = false) => teclas.forEach(f => f({ key, repeat, altKey: false, ctrlKey: false,
    preventDefault() {} })) };
}

const espera = () => new Promise(r => setImmediate(r));
let falhas = 0;
const confere = (ok, msg) => { console.log(`  ${ok ? 'ok  ' : 'FALHA:'} ${msg}`); if (!ok) falhas++; };

// a barra com duas telas, cada uma com a sua tinta; a ponte anota cada "limpar" e em que tela foi
function barra() {
  const tinta = { A: 3, B: 2 }, limpou = [];
  let atual = 'A';
  const telas = () => ({ telas: [{ nome: 'A' }, { nome: 'B' }], atual: atual === 'A' ? 1 : 2 });
  const ponte = {
    camada: async (nome, valor) => {
      if (nome === 'tela') atual = valor;
      if (nome === 'limpar') { limpou.push(atual); tinta[atual] = 0; }
      return { tracos: tinta[atual] };
    },
    telas: async () => telas(),
    mudarTela: async (n) => { atual = n === 1 ? 'A' : 'B'; return telas(); },
    tecla() {}, fixar: async v => v, recolher: async () => {}, preferencias: async () => {},
  };
  const p = pagina('barra.html', ponte);
  return { ...p, limpou };
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

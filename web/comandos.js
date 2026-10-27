// Giz na Tela — © 2026 Heraldo Antunes — Licença MIT (ver LICENSE). Registro único de comandos
// Cada ação do anotador é um comando com nome, descrição, nível e parâmetros. Botões, atalhos, Esc, a API local e o
// MCP chegam todos por aqui: a barra executa (ela guarda o estado dos botões) e a ajuda lê esta lista para mostrar
// os atalhos. Comando novo entra primeiro aqui; o teste confere que cada um tem quem o execute e um teste próprio.
(() => {
const FERRAMENTAS = ['caneta', 'marca', 'borracha', 'reta', 'seta', 'retangulo', 'elipse', 'texto', 'numero', 'visto',
  'xis', 'holofote', 'lupa'];
const NOME_FERRAMENTA = { caneta: 'Caneta', marca: 'Marca-texto', borracha: 'Borracha', reta: 'Reta', seta: 'Seta',
  retangulo: 'Retângulo', elipse: 'Elipse', texto: 'Texto', numero: 'Numeração', visto: 'Visto', xis: 'X',
  holofote: 'Holofote', lupa: 'Lupa' };
const CORES = [['#e53935', 'Vermelho'], ['#1e6fd9', 'Azul'], ['#2e9d4a', 'Verde'], ['#f9c80e', 'Amarelo'],
  ['#111111', 'Preto'], ['#ffffff', 'Branco']];
const ESPESSURAS = { fina: 2, media: 4, grossa: 8 };

// tipos de parâmetro: a mesma validação vale para o clique, o atalho e o pedido de fora (que não é confiável)
const X = { tipo: 'numero', min: -20000, max: 20000, descricao: 'px a partir da esquerda da tela' };
const Y = { tipo: 'numero', min: -20000, max: 20000, descricao: 'px a partir do topo da tela' };
const COR = { tipo: 'cor', opcional: true, descricao: 'cor #rrggbb (vazio: a cor escolhida na barra)' };
const ESP = { tipo: 'texto', valores: Object.keys(ESPESSURAS), opcional: true, descricao: 'fina, media ou grossa (vazio: a da barra)' };
const LIGADO = descricao => ({ tipo: 'logico', opcional: true, descricao: descricao + ' (vazio: alterna)' });

// nivel: 'ler' só consulta; 'agir' muda a tela. desfaz: entra no Desfazer como um passo só.
// volta_a_escrever: pelo botão ou atalho, sai do "Mouse livre" (escolher algo de desenho é querer desenhar)
const COMANDOS = [
  { nome: 'ferramenta.escolher', nivel: 'agir', volta_a_escrever: true, aviso: p => `escolheu ${NOME_FERRAMENTA[p.ferramenta]}`,
    descricao: 'Escolhe a ferramenta: caneta, marca (marca-texto), borracha, reta, seta, retangulo, elipse, texto, ' +
      'numero, visto, xis, holofote (escurece em volta do ponteiro) ou lupa.',
    parametros: { ferramenta: { tipo: 'texto', valores: FERRAMENTAS } } },
  { nome: 'cor.escolher', nivel: 'agir', volta_a_escrever: true, aviso: () => 'mudou a cor',
    descricao: 'Escolhe a cor da tinta (#rrggbb). Com a borracha escolhida, volta para a caneta.',
    parametros: { cor: { tipo: 'cor' } } },
  { nome: 'espessura.escolher', nivel: 'agir', volta_a_escrever: true, aviso: p => `escolheu o traço ${p.espessura}`,
    descricao: 'Escolhe a espessura do traço: fina, media ou grossa (também muda o tamanho do texto e dos carimbos).',
    parametros: { espessura: { tipo: 'texto', valores: Object.keys(ESPESSURAS) } } },
  { nome: 'desenho.desfazer', nivel: 'agir', aviso: () => 'desfez um passo', descricao: 'Desfaz o último passo.' },
  { nome: 'desenho.refazer', nivel: 'agir', aviso: () => 'refez um passo', descricao: 'Refaz o passo desfeito.' },
  { nome: 'desenho.limpar', nivel: 'agir', desfaz: true, aviso: () => 'limpou a tela',
    descricao: 'Apaga toda a tinta da tela (dá para desfazer).' },
  { nome: 'quadro.alternar', nivel: 'agir', volta_a_escrever: true, aviso: p => `quadro: ${p.quadro || 'próximo'}`,
    descricao: 'Cobre a tela com um quadro liso: branco, negro, ou tela (volta a mostrar os programas). Sem ' +
      'parâmetro, passa para o próximo (tela, branco, negro).',
    parametros: { quadro: { tipo: 'texto', valores: ['tela', 'branco', 'negro'], opcional: true } } },
  { nome: 'forma.preencher', nivel: 'agir', volta_a_escrever: true, aviso: p => `preenchimento: ${p.modo || 'próximo'}`,
    descricao: 'Preenchimento do retângulo e da elipse: nada (só contorno), translucido (destaca) ou cheio (tapa o ' +
      'que está embaixo). Sem parâmetro, passa para o próximo.',
    parametros: { modo: { tipo: 'texto', valores: ['nada', 'translucido', 'cheio'], opcional: true } } },
  { nome: 'tinta.some', nivel: 'agir', volta_a_escrever: true, aviso: () => 'mudou a tinta que some',
    descricao: 'Liga ou desliga a tinta que some sozinha em 3 segundos (para apontar sem sujar a tela).',
    parametros: { ligado: LIGADO('true liga') } },
  { nome: 'tela.congelar', nivel: 'agir', volta_a_escrever: true, aviso: () => 'congelou ou descongelou a tela',
    descricao: 'Congela a tela (um retrato parado do que está embaixo, para explicar um vídeo ou animação).',
    parametros: { ligado: LIGADO('true congela') } },
  { nome: 'tela.ampliar', nivel: 'agir', volta_a_escrever: true, aviso: () => 'mudou o aumento',
    descricao: 'Ampliar: ligar (o próximo toque escolhe o ponto, 2 vezes), desligar, mais ou menos (no meio da tela). ' +
      'Sem parâmetro, liga ou desliga.',
    parametros: { modo: { tipo: 'texto', valores: ['ligar', 'desligar', 'mais', 'menos'], opcional: true } } },
  { nome: 'tela.fixar', nivel: 'agir', aviso: p => p.ligado === false ? 'voltou a escrever' : 'soltou o mouse',
    descricao: 'Modo "Mouse livre" (ligado: o clique vai para os outros programas e a tinta fica na tela) ou ' +
      '"Escrevendo" (desligado).',
    parametros: { ligado: LIGADO('true = Mouse livre') } },
  { nome: 'barra.recolher', nivel: 'agir', aviso: () => 'recolheu ou abriu a barra',
    descricao: 'Recolhe a barra num botão pequeno (o mouse fica livre e a tinta continua) ou abre de novo.',
    parametros: { recolhida: LIGADO('true recolhe') } },
  { nome: 'captura.salvar', nivel: 'agir', aviso: () => 'salvou a tela como imagem',
    descricao: 'Salva a tela com a tinta como PNG em Documentos\\Giz na Tela\\capturas (a barra não aparece).' },
  { nome: 'captura.mandar_ao_giz', nivel: 'agir', desfaz: true, aviso: () => 'mandou a tela ao Giz Livre',
    descricao: 'Manda a tela com a tinta para o Giz Livre como quadro novo (abre o Giz se estiver fechado). ' +
      'Depois a tinta sai da tela (dá para desfazer) e a barra recolhe.' },
  { nome: 'captura.ler', nivel: 'ler',
    descricao: 'Devolve um retrato da tela com a tinta, em PNG (base64), sem a barra. largura_max reduz a imagem.',
    parametros: { largura_max: { tipo: 'numero', min: 160, max: 3840, inteiro: true, opcional: true,
      descricao: 'largura máxima em px (padrão 1280)' } } },
  { nome: 'texto.escrever', nivel: 'agir', desfaz: true, aviso: () => 'escreveu um texto',
    descricao: 'Escreve um texto na tela (\\n quebra a linha), com o canto de cima à esquerda em x, y.',
    parametros: { texto: { tipo: 'texto', min: 1, max: 500 }, x: X, y: Y, cor: COR,
      tamanho: { tipo: 'numero', min: 10, max: 200, opcional: true, descricao: 'px (vazio: conforme a espessura)' } } },
  { nome: 'numero.carimbar', nivel: 'agir', desfaz: true, aviso: () => 'carimbou um número',
    descricao: 'Carimba um número num círculo com o centro em x, y (vazio: o próximo da sequência).',
    parametros: { x: X, y: Y, numero: { tipo: 'numero', min: 1, max: 999, inteiro: true, opcional: true }, cor: COR } },
  { nome: 'correcao.carimbar', nivel: 'agir', desfaz: true, aviso: p => `carimbou um ${p.marca === 'xis' ? 'X' : 'visto'}`,
    descricao: 'Carimba um visto (✓) ou um X (marca: visto ou xis) com o centro em x, y.',
    parametros: { marca: { tipo: 'texto', valores: ['visto', 'xis'] }, x: X, y: Y, cor: COR } },
  { nome: 'forma.desenhar', nivel: 'agir', desfaz: true, aviso: () => 'desenhou uma forma',
    descricao: 'Desenha reta, seta, retangulo ou elipse de (x1, y1) até (x2, y2). A seta aponta para (x2, y2).',
    parametros: { forma: { tipo: 'texto', valores: ['reta', 'seta', 'retangulo', 'elipse'] },
      x1: X, y1: Y, x2: X, y2: Y, cor: COR, espessura: ESP,
      preencher: { tipo: 'texto', valores: ['nada', 'translucido', 'cheio'], opcional: true } } },
  { nome: 'traco.desenhar', nivel: 'agir', desfaz: true, aviso: () => 'desenhou um traço',
    descricao: 'Desenha um traço livre pelos pontos [x, y] ou [x, y, pressão de 0 a 1]. marca: true faz marca-texto.',
    parametros: { pontos: { tipo: 'pontos', min: 2, max: 5000 }, cor: COR, espessura: ESP,
      marca: { tipo: 'logico', opcional: true } } },
  { nome: 'desenho.adicionar', nivel: 'agir', desfaz: true, aviso: p => `desenhou ${p.itens.length} ${p.itens.length === 1 ? 'item' : 'itens'}`,
    descricao: 'Desenha vários itens de uma vez, num passo só do Desfazer. Cada item tem `tipo` (texto, numero, ' +
      'correcao, forma ou traco) e os campos do comando de um item (texto.escrever, numero.carimbar, ' +
      'correcao.carimbar, forma.desenhar, traco.desenhar). Ou todos entram, ou nenhum.',
    parametros: { itens: { tipo: 'itens', min: 1, max: 100 } } },
  { nome: 'estado.ler', nivel: 'ler',
    descricao: 'Estado do anotador: ferramenta, cor, espessura, modo, quadro, congelado, ampliado, quantidade de ' +
      'desenhos e tamanho da tela.' },
  { nome: 'comandos.listar', nivel: 'ler', descricao: 'Lista os comandos, os atalhos de teclado e os botões da barra.' },
  { nome: 'modo.voltar_um_passo', nivel: 'agir', aviso: () => 'voltou um passo',
    descricao: 'O que o Esc faz: desliga o ampliar, depois o congelar, depois o quadro, volta para a caneta e, já ' +
      'na caneta, solta o mouse. Nunca fecha o anotador.' },
  { nome: 'ajuda.abrir', nivel: 'agir', aviso: () => 'abriu a ajuda', descricao: 'Abre a janela de ajuda e ajustes.' },
  { nome: 'programa.fechar', nivel: 'agir',
    descricao: 'Fecha o anotador (os desenhos se perdem). Com desenho na tela, só fecha com confirmar: true; pelo ' +
      'botão, o primeiro toque pede um segundo toque para confirmar.',
    parametros: { confirmar: { tipo: 'logico', opcional: true } } },
];

// atalhos de teclado: valem com o foco na barra ou na camada (Ctrl+Alt+F vale de qualquer programa)
const ATALHOS = {
  c: ['ferramenta.escolher', { ferramenta: 'caneta' }], m: ['ferramenta.escolher', { ferramenta: 'marca' }],
  b: ['ferramenta.escolher', { ferramenta: 'borracha' }], l: ['ferramenta.escolher', { ferramenta: 'reta' }],
  s: ['ferramenta.escolher', { ferramenta: 'seta' }], r: ['ferramenta.escolher', { ferramenta: 'retangulo' }],
  e: ['ferramenta.escolher', { ferramenta: 'elipse' }], t: ['ferramenta.escolher', { ferramenta: 'texto' }],
  n: ['ferramenta.escolher', { ferramenta: 'numero' }], v: ['ferramenta.escolher', { ferramenta: 'visto' }],
  x: ['ferramenta.escolher', { ferramenta: 'xis' }], h: ['ferramenta.escolher', { ferramenta: 'holofote' }],
  u: ['ferramenta.escolher', { ferramenta: 'lupa' }],
  1: ['cor.escolher', { cor: CORES[0][0] }], 2: ['cor.escolher', { cor: CORES[1][0] }],
  3: ['cor.escolher', { cor: CORES[2][0] }], 4: ['cor.escolher', { cor: CORES[3][0] }],
  5: ['cor.escolher', { cor: CORES[4][0] }], 6: ['cor.escolher', { cor: CORES[5][0] }],
  7: ['espessura.escolher', { espessura: 'fina' }], 8: ['espessura.escolher', { espessura: 'media' }],
  9: ['espessura.escolher', { espessura: 'grossa' }],
  a: ['tela.ampliar', {}], '+': ['tela.ampliar', { modo: 'mais' }], '=': ['tela.ampliar', { modo: 'mais' }],
  '-': ['tela.ampliar', { modo: 'menos' }], g: ['tela.congelar', {}], q: ['quadro.alternar', {}],
  p: ['forma.preencher', {}], d: ['tinta.some', {}], f: ['tela.fixar', {}], o: ['barra.recolher', { recolhida: true }],
  escape: ['modo.voltar_um_passo', {}], f1: ['ajuda.abrir', {}],
  'ctrl+z': ['desenho.desfazer', {}], 'ctrl+y': ['desenho.refazer', {}], 'ctrl+s': ['captura.salvar', {}],
  'ctrl+g': ['captura.mandar_ao_giz', {}],
};
const ATALHO_GLOBAL = { tecla: 'Ctrl+Alt+F', comando: 'tela.fixar' };

const POR_NOME = Object.fromEntries(COMANDOS.map(c => [c.nome, c]));
const ITEM_COMANDO = { texto: 'texto.escrever', numero: 'numero.carimbar', correcao: 'correcao.carimbar',
  forma: 'forma.desenhar', traco: 'traco.desenhar' };

function falha(msg) { throw new Error(msg); }
function validarValor(nome, d, v) {
  switch (d.tipo) {
    case 'texto':
      if (typeof v !== 'string') falha(`${nome}: precisa ser texto`);
      if (d.valores && !d.valores.includes(v)) falha(`${nome}: use ${d.valores.join(', ')}`);
      if (v.length < (d.min ?? 0) || v.length > (d.max ?? 200)) falha(`${nome}: de ${d.min ?? 0} a ${d.max ?? 200} caracteres`);
      if (d.min && !v.trim()) falha(`${nome}: não pode ficar em branco`);
      return v;
    case 'numero':
      if (typeof v !== 'number' || !Number.isFinite(v)) falha(`${nome}: precisa ser número`);
      if (d.inteiro && !Number.isInteger(v)) falha(`${nome}: precisa ser inteiro`);
      if (v < d.min || v > d.max) falha(`${nome}: de ${d.min} a ${d.max}`);
      return v;
    case 'logico':
      if (typeof v !== 'boolean') falha(`${nome}: true ou false`);
      return v;
    case 'cor':
      if (typeof v !== 'string' || !/^#[0-9a-fA-F]{6}$/.test(v)) falha(`${nome}: cor no formato #rrggbb`);
      return v.toLowerCase();
    case 'pontos': {
      if (!Array.isArray(v) || v.length < d.min || v.length > d.max) falha(`${nome}: lista de ${d.min} a ${d.max} pontos`);
      return v.map((p, i) => {
        if (!Array.isArray(p) || p.length < 2 || p.length > 3) falha(`${nome}[${i}]: use [x, y] ou [x, y, pressão]`);
        const [x, y, pr = 0.5] = p;
        return [validarValor(`${nome}[${i}].x`, X, x), validarValor(`${nome}[${i}].y`, Y, y),
          validarValor(`${nome}[${i}].pressão`, { tipo: 'numero', min: 0, max: 1 }, pr)];
      });
    }
    case 'itens': {
      if (!Array.isArray(v) || v.length < d.min || v.length > d.max) falha(`${nome}: lista de ${d.min} a ${d.max} itens`);
      return v.map((item, i) => {
        if (!item || typeof item !== 'object' || Array.isArray(item)) falha(`${nome}[${i}]: precisa ser um objeto`);
        const { tipo, ...resto } = item, cmd = ITEM_COMANDO[tipo];
        if (!cmd) falha(`${nome}[${i}].tipo: use ${Object.keys(ITEM_COMANDO).join(', ')}`);
        try { return { tipo, ...validar(cmd, resto) }; } catch (e) { falha(`${nome}[${i}] (${tipo}): ${e.message}`); }
      });
    }
  }
  falha(`${nome}: tipo desconhecido`);
}
// devolve só os parâmetros conhecidos e válidos; parâmetro a mais é erro (pega erro de digitação do agente)
function validar(nomeComando, args) {
  const c = POR_NOME[nomeComando];
  if (!c) falha(`comando desconhecido: ${nomeComando} (veja comandos.listar)`);
  if (args == null) args = {};
  if (typeof args !== 'object' || Array.isArray(args)) falha('parâmetros: precisa ser um objeto');
  const defs = c.parametros || {}, saida = {};
  for (const k of Object.keys(args)) if (!(k in defs)) falha(`${nomeComando}: parâmetro desconhecido "${k}"`);
  for (const [k, d] of Object.entries(defs)) {
    if (args[k] === undefined || args[k] === null) { if (!d.opcional) falha(`${nomeComando}: falta "${k}"`); continue; }
    saida[k] = validarValor(k, d, args[k]);
  }
  return saida;
}
// descrição para quem está de fora (API, MCP, ajuda): sem as funções internas
function descrever(c) {
  return { nome: c.nome, descricao: c.descricao, nivel: c.nivel, desfaz: !!c.desfaz,
    parametros: Object.fromEntries(Object.entries(c.parametros || {}).map(([k, d]) => [k, { ...d }])) };
}

window.GizTela = { COMANDOS, POR_NOME, ATALHOS, ATALHO_GLOBAL, CORES, ESPESSURAS, FERRAMENTAS, NOME_FERRAMENTA, validar, descrever };
})();

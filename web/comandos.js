// Giz na Tela — © 2026 Heraldo Antunes — Licença MIT (ver LICENSE). Registro único de comandos
// Cada ação do anotador é um comando com nome, descrição, nível e parâmetros. Botões, atalhos, Esc, a API local e o
// MCP chegam todos por aqui: a barra executa (ela guarda o estado dos botões) e a ajuda lê esta lista para mostrar
// os atalhos. Comando novo entra primeiro aqui; o teste confere que cada um tem quem o execute e um teste próprio.
(() => {
const FERRAMENTAS = ['caneta', 'marca', 'borracha', 'mover', 'reta', 'seta', 'cota', 'medir', 'retangulo', 'elipse', 'texto', 'numero', 'visto',
  'xis', 'holofote', 'lupa'];
const NOME_FERRAMENTA = { caneta: 'Caneta', marca: 'Marca-texto', borracha: 'Borracha', mover: 'Mover (mão)', reta: 'Reta', seta: 'Seta', cota: 'Cota', medir: 'Medir',
  retangulo: 'Retângulo', elipse: 'Elipse', texto: 'Texto', numero: 'Numeração', visto: 'Visto', xis: 'X',
  holofote: 'Holofote', lupa: 'Lupa' };
// cores de fábrica da barra, na ordem das bolinhas (o verde é a principal); o professor pode trocar cada uma
const CORES = [['#2e9d4a', 'Verde'], ['#e53935', 'Vermelho'], ['#1e6fd9', 'Azul'], ['#f9c80e', 'Amarelo'],
  ['#111111', 'Preto'], ['#ffffff', 'Branco']];
const POSICAO = { tipo: 'numero', min: 1, max: CORES.length, inteiro: true, descricao: 'bolinha da barra (1 a 6)' };
// aparência da cota (setas ou tracinhos de arquitetura); medida digitada e medida automática são do jeito de desenhar
const COTAS = ['setas', 'tracos'];
const TIPOS_DE_COTA = ['setas', 'medida', 'auto', 'tracos'];
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
    descricao: 'Escolhe a ferramenta: caneta, marca (marca-texto), borracha, mover (a mão: arrasta um objeto), reta, seta, cota ' +
      '(barras nas pontas; ao soltar, digita-se a medida), medir (cota que mede sozinha), retangulo, elipse, texto, ' +
      'numero, visto, xis, holofote (escurece em volta do ponteiro) ou lupa.',
    parametros: { ferramenta: { tipo: 'texto', valores: FERRAMENTAS } } },
  { nome: 'cor.escolher', nivel: 'agir', volta_a_escrever: true, aviso: () => 'mudou a cor',
    descricao: 'Escolhe a cor da tinta (#rrggbb). Com a borracha escolhida, volta para a caneta.',
    parametros: { cor: { tipo: 'cor' } } },
  { nome: 'cor.posicao', nivel: 'agir', volta_a_escrever: true, aviso: p => `escolheu a cor ${p.posicao}`,
    descricao: 'Escolhe a cor de uma das bolinhas da barra (1 a 6), com a cor que ela tiver agora.',
    parametros: { posicao: POSICAO } },
  { nome: 'cor.trocar', nivel: 'agir', volta_a_escrever: true, aviso: p => `trocou a cor ${p.posicao}`,
    descricao: 'Troca a cor de uma bolinha da barra e já a escolhe. Pelo botão (segundo toque na bolinha escolhida), ' +
      'abre o seletor de cores do Windows; de fora, mande a cor (#rrggbb).',
    parametros: { posicao: POSICAO, cor: { tipo: 'cor', opcional: true } } },
  { nome: 'cores.restaurar', nivel: 'agir', aviso: () => 'voltou às cores de fábrica',
    descricao: 'Volta as bolinhas da barra às cores de fábrica (verde, vermelho, azul, amarelo, preto e branco).' },
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
  { nome: 'seta.estilo', nivel: 'agir', volta_a_escrever: true, aviso: () => 'mudou o tipo de seta',
    descricao: 'Tipo das setas novas: ponta aberta (em V, a de fábrica) ou cheia (triângulo), simples ou dupla (ponta ' +
      'nos dois lados). Sem parâmetro, passa para o próximo: simples aberta, simples cheia, dupla aberta, dupla cheia. ' +
      'Pelo botão, é cada toque a mais na seta já escolhida (ou S de novo).',
    parametros: { estilo: { tipo: 'texto', valores: ['aberta', 'cheia'], opcional: true },
      dupla: { tipo: 'logico', opcional: true } } },
  { nome: 'cota.estilo', nivel: 'agir', volta_a_escrever: true, aviso: p => `cota: ${p.estilo || 'próximo tipo'}`,
    descricao: 'Aparência das cotas novas (da Cota e do Medir): setas (barras nas pontas e setas até elas) ou tracos ' +
      '(tracinhos inclinados, como em arquitetura). Sem parâmetro, alterna. Pelo botão, é o segundo toque na cota já ' +
      'escolhida (ou K de novo).',
    parametros: { estilo: { tipo: 'texto', valores: COTAS, opcional: true } } },
  { nome: 'cota.escala', nivel: 'agir', aviso: p => (p.valor ? `régua: ${p.valor} ${p.unidade}` : 'tirou a régua'),
    descricao: 'Régua da cota automática: pixels px medem valor na unidade (ex.: valor 10, unidade cm, pixels 245). ' +
      'Sem parâmetros, tira a régua (a cota automática volta a medir em px). Pelo uso normal, a régua se define ' +
      'sozinha quando o professor digita uma medida com unidade (ex.: 10 cm) numa cota com medida.',
    parametros: { valor: { tipo: 'numero', min: 0.000001, max: 1e9, opcional: true },
      unidade: { tipo: 'texto', valores: ['mm', 'cm', 'm', 'km'], opcional: true },
      pixels: { tipo: 'numero', min: 1, max: 20000, opcional: true } } },
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
    descricao: 'Desenha reta, seta, cota, retangulo ou elipse de (x1, y1) até (x2, y2). A seta aponta para (x2, y2). ' +
      'Na cota, medida é o texto no meio da linha (ex.: 2,5 m).',
    parametros: { forma: { tipo: 'texto', valores: ['reta', 'seta', 'cota', 'retangulo', 'elipse'] },
      x1: X, y1: Y, x2: X, y2: Y, cor: COR, espessura: ESP,
      preencher: { tipo: 'texto', valores: ['nada', 'translucido', 'cheio'], opcional: true },
      ponta: { tipo: 'texto', valores: ['aberta', 'cheia'], opcional: true, descricao: 'só para seta (vazio: a da barra)' },
      dupla: { tipo: 'logico', opcional: true, descricao: 'só para seta: ponta nos dois lados (vazio: a da barra)' },
      cota: { tipo: 'texto', valores: TIPOS_DE_COTA, opcional: true, descricao: 'só para cota (vazio: a da barra); auto mede sozinha' },
      medida: { tipo: 'texto', min: 1, max: 40, opcional: true, descricao: 'só para cota: texto no meio da linha' } } },
  { nome: 'traco.desenhar', nivel: 'agir', desfaz: true, aviso: () => 'desenhou um traço',
    descricao: 'Desenha um traço livre pelos pontos [x, y] ou [x, y, pressão de 0 a 1]. marca: true faz marca-texto.',
    parametros: { pontos: { tipo: 'pontos', min: 2, max: 5000 }, cor: COR, espessura: ESP,
      marca: { tipo: 'logico', opcional: true } } },
  { nome: 'objeto.mover', nivel: 'agir', desfaz: true, aviso: () => 'moveu um objeto',
    descricao: 'Move o objeto que estiver no ponto (x, y) (o de cima, se houver vários) dx px para a direita e dy px ' +
      'para baixo. É o que a mão faz ao arrastar.',
    parametros: { x: X, y: Y, dx: { tipo: 'numero', min: -20000, max: 20000 }, dy: { tipo: 'numero', min: -20000, max: 20000 } } },
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
  s: ['ferramenta.escolher', { ferramenta: 'seta' }], k: ['ferramenta.escolher', { ferramenta: 'cota' }],
  j: ['ferramenta.escolher', { ferramenta: 'medir' }], ' ': ['ferramenta.escolher', { ferramenta: 'mover' }], r: ['ferramenta.escolher', { ferramenta: 'retangulo' }],
  e: ['ferramenta.escolher', { ferramenta: 'elipse' }], t: ['ferramenta.escolher', { ferramenta: 'texto' }],
  n: ['ferramenta.escolher', { ferramenta: 'numero' }], v: ['ferramenta.escolher', { ferramenta: 'visto' }],
  x: ['ferramenta.escolher', { ferramenta: 'xis' }], h: ['ferramenta.escolher', { ferramenta: 'holofote' }],
  u: ['ferramenta.escolher', { ferramenta: 'lupa' }],
  1: ['cor.posicao', { posicao: 1 }], 2: ['cor.posicao', { posicao: 2 }], 3: ['cor.posicao', { posicao: 3 }],
  4: ['cor.posicao', { posicao: 4 }], 5: ['cor.posicao', { posicao: 5 }], 6: ['cor.posicao', { posicao: 6 }],
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
// tocar de novo (ou a mesma tecla) numa ferramenta já escolhida troca uma opção dela, sem botão a mais na barra
const DE_NOVO = { seta: ['seta.estilo', {}], cota: ['cota.estilo', {}] };

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

window.GizTela = { COTAS, COMANDOS, POR_NOME, ATALHOS, ATALHO_GLOBAL, DE_NOVO, CORES, ESPESSURAS, FERRAMENTAS, NOME_FERRAMENTA, validar, descrever };
})();

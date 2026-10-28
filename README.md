<p align="center"><img src="docs/img/icone.png" alt="" width="120"></p>

<h1 align="center">Giz na Tela <img src="docs/img/bandeira-brasil.svg" alt="Brasil" title="Feito no Brasil" width="20" height="14"></h1>

<p align="center"><b>Escrever por cima de qualquer programa: slides, PDF, navegador, vídeo, simulação.</b><br>
Uma camada transparente e uma barra pequena com caneta, formas, texto, holofote e lupa, feitas para a mesa
digitalizadora. Livre, gratuito e sem internet.</p>

<p align="center">
  <img src="https://img.shields.io/badge/feito%20em-Rust-555" alt="Feito em Rust">
  <a href="#requisitos"><img src="https://img.shields.io/badge/testado%20no-Windows%2011-0078D6" alt="Testado no Windows 11"></a>
  <img src="https://img.shields.io/badge/internet-n%C3%A3o%20precisa-2b8a3e" alt="Não precisa de internet">
  <a href="LICENSE"><img src="https://img.shields.io/badge/licen%C3%A7a-MIT-blue" alt="Licença MIT"></a>
  <img src="https://img.shields.io/badge/situa%C3%A7%C3%A3o-em%20teste-e8590c" alt="Situação: em teste">
  <a href="#comandos-para-tudo-atalhos-testes-e-agentes-de-ia"><img src="https://img.shields.io/badge/agentes-MCP-6f42c1" alt="Agentes: MCP"></a>
  <br>
  <a href="https://github.com/HeraldoAntunes/giz-na-tela/actions/workflows/testes.yml"><img src="https://github.com/HeraldoAntunes/giz-na-tela/actions/workflows/testes.yml/badge.svg" alt="Testes"></a>
  <a href="https://scorecard.dev/viewer/?uri=github.com/HeraldoAntunes/giz-na-tela"><img src="https://api.scorecard.dev/projects/github.com/HeraldoAntunes/giz-na-tela/badge" alt="OpenSSF Scorecard"></a>
  <a href="https://github.com/HeraldoAntunes/giz-na-tela/releases/latest"><img src="https://img.shields.io/github/v/release/HeraldoAntunes/giz-na-tela?label=vers%C3%A3o" alt="Versão"></a>
  <a href="https://github.com/HeraldoAntunes/giz-na-tela/releases"><img src="https://img.shields.io/github/downloads/HeraldoAntunes/giz-na-tela/total?label=downloads" alt="Downloads"></a>
  <a href="https://doi.org/10.5281/zenodo.23690523"><img src="https://zenodo.org/badge/DOI/10.5281/zenodo.23690523.svg" alt="DOI"></a>
</p>

<p align="center">
  <a href="https://github.com/HeraldoAntunes/giz-na-tela/releases/latest"><b>⬇ Baixar o Giz na Tela</b></a>
  &nbsp;·&nbsp; <a href="#como-se-usa">Guia de uso</a>
  &nbsp;·&nbsp; <a href="#programas-livres-">Programas Livres</a>
</p>

<p align="center"><img src="docs/img/capa.png" alt="Giz na Tela: a barra de ferramentas por cima de uma apostila em PDF anotada com marca-texto, números, seta, texto e visto" width="820"></p>

<p align="center">
  <a href="#qual-arquivo-baixar">Baixar</a> · <a href="#como-se-usa">Como se usa</a> · <a href="#o-que-faz">O que faz</a> ·
  <a href="#requisitos">Requisitos</a> · <a href="#perguntas-frequentes">Perguntas</a> · <a href="#programas-livres-">Programas Livres</a>
</p>

## Qual arquivo baixar?

Versão de agora: **1.3.1** ([todas as versões](https://github.com/HeraldoAntunes/giz-na-tela/releases)).

| Para quê | Baixar |
|---|---|
| **Instalar no computador (Windows)** | [**Instalar-GizNaTela-1.3.1.exe**](https://github.com/HeraldoAntunes/giz-na-tela/releases/download/v1.3.1/Instalar-GizNaTela-1.3.1.exe) |
| Usar sem instalar (versão portátil, para pendrive) | [GizNaTela-1.3.1-portatil.zip](https://github.com/HeraldoAntunes/giz-na-tela/releases/download/v1.3.1/GizNaTela-1.3.1-portatil.zip) |

Se o Windows avisar "o Windows protegeu o computador", clique em **Mais informações → Executar assim mesmo** (o
programa é gratuito e não tem assinatura paga). A conferência dos arquivos fica na
[página de versões](https://github.com/HeraldoAntunes/giz-na-tela/releases).

- **Grátis e livre:** licença [MIT](LICENSE). Use, copie e distribua à vontade, inclusive em escolas e redes de ensino.
- **Sem internet, sem conta, sem anúncios:** nada sai do computador; só a procura de versão nova, quando você clica,
  consulta o GitHub.
- **Pequeno:** um programa só, de cerca de 3,5 MB, sem nada a mais para instalar no Windows 11.

> **Aviso:** projeto independente e voluntário, fornecido "no estado em que se encontra", **sem garantia de qualquer
> tipo**. As marcas citadas pertencem aos seus donos; o Giz na Tela é um projeto independente, sem vínculo com elas.

## Como se usa

<p align="center"><img src="docs/img/uso-slide.png" alt="Slide de física anotado com o Giz na Tela: título destacado com marca-texto, itens numerados, fórmula circulada com visto, seta até o bloco e texto explicativo; a barra de ferramentas fica à direita" width="860"></p>

O botão grande da barra diz o modo de agora. **Escrevendo:** a caneta desenha por cima de tudo. **Mouse livre:** o
clique vai para o programa de baixo (passar o slide, dar play no vídeo) e a tinta continua na tela. De qualquer
programa, **Ctrl+Alt+F** volta a escrever. **Esc** volta um passo de cada vez e nunca fecha nem apaga.

<table>
<tr>
<td width="50%"><img src="docs/img/uso-recolhida.png" alt="A barra recolhida num botão verde pequeno no canto, com a tinta continuando sobre o slide"><br>
<b>Barra recolhida:</b> vira um botão verde que se arrasta; o mouse fica livre e a tinta continua. O programa lembra
onde a barra ficou.</td>
<td width="50%"><img src="docs/img/uso-holofote.png" alt="Holofote: a tela escurecida, com um círculo de luz sobre o bloco do plano inclinado"><br>
<b>Holofote:</b> escurece tudo em volta do ponteiro, para a turma olhar onde importa. A roda do mouse muda o tamanho.</td>
</tr>
<tr>
<td><img src="docs/img/uso-quadro-negro.png" alt="Quadro negro por cima de tudo, com a resolução de uma conta e o resultado circulado"><br>
<b>Quadro branco ou negro</b> por cima de tudo, para resolver uma conta sem sair do slide.</td>
<td><img src="docs/img/uso-ajuda.png" alt="Janela de ajuda em português aberta sobre o slide"><br>
<b>Ajuda em português</b> (tecla F1 ou o "?" da barra), com todos os atalhos.</td>
</tr>
</table>

## O que faz

- **Caneta com pressão** da mesa digitalizadora, marca-texto e borracha (inclusive a ponta de apagar da caneta).
- **Formas:** reta, seta, retângulo e elipse (Shift: 15 em 15 graus, quadrado, círculo), com preenchimento
  translúcido (destaca) ou cheio (tapa a resposta).
- **Texto, numeração (1, 2, 3...), visto e X** para corrigir exercícios na tela.
- **Laço:** contorne vários traços, formas e textos e arraste o grupo, troque a cor dele ou apague de uma vez. A
  **mão** move um objeto só.
- **Canetas prontas:** três canetas com a sua cor, espessura e tipo (caneta ou marca-texto), trocadas num toque.
- **Shift** deixa reto o traço da caneta e do marca-texto.
- **Holofote e lupa.**
- **Congelar a tela** (um vídeo parado enquanto você explica) e **ampliar** um ponto para escrever em cima.
- **Tinta que some** sozinha em 3 segundos, para apontar sem sujar a tela.
- **Projetor no modo estender:** um toque leva a tinta e a barra para o projetor e traz de volta; cada tela guarda a
  sua tinta.
- **Copiar** a tela com a tinta (Ctrl+C) para colar num documento ou numa prova, **salvar** como imagem
  (`Documentos\Giz na Tela\capturas`) ou **mandar ao
  [Giz Livre](https://github.com/HeraldoAntunes/giz-livre)** como quadro novo (abre o Giz se estiver fechado).
- **Fechar com desenho na tela** pede um segundo toque no X, para não perder a aula sem querer.
- **Botão lateral da caneta** (experimental): enquanto apertado, vira borracha, marca-texto, reta, mão ou laço.
- Seis cores, três espessuras e atalhos de teclado para tudo (lista completa na ajuda).

## Comandos para tudo: atalhos, testes e agentes de IA

Cada ação da barra é um comando com nome, descrição e parâmetros (`web/comandos.js`). O botão, o atalho de teclado,
o teste automático e o agente de IA usam o mesmo caminho, então o que um faz o outro também faz.

Para assistentes de IA compatíveis com MCP, o próprio programa é o servidor (`GizNaTela.exe --mcp`), falando com uma
API local que só aceita programas do mesmo computador, com um código de acesso que o programa cria. **Vem desligado.**
Na ajuda (aba Agentes de IA) o professor escolhe:

- **Só ler:** o agente vê o estado e um retrato da tela com as anotações;
- **Agir:** o agente também escreve, desenha, aponta e congela a tela. Cada ação aparece com o aviso "Agente: ...", e
  o que ele desenha sai com um Ctrl+Z. Com desenho na tela, ele não fecha o anotador.

A mesma aba mostra o trecho pronto para a configuração do assistente. Detalhes de segurança no
[SECURITY.md](.github/SECURITY.md).

## Requisitos

- Windows 10 ou 11, 64 bits. Não precisa de administrador (instala só para a sua conta, ou para todos, se quiser).
- O WebView2 da Microsoft, que desenha a barra e a tinta. Já vem no Windows 11 e nos Windows 10 atualizados; se
  faltar, o instalador o instala junto (precisa de internet nessa hora) e a versão portátil explica onde baixar.
- Mesa digitalizadora: para a pressão funcionar, ligue **"Usar Windows Ink"** no painel de ajustes da mesa. Testado no
  Windows 11 com mesa digitalizadora de caneta.

Com projetor no modo "estender", a tinta fica numa tela por vez: o botão de tela da barra (tecla Y) troca de uma para
a outra. Escrever nas duas ao mesmo tempo ainda não dá.

## Perguntas frequentes

**É de graça mesmo?** Sim. É software livre (licença MIT): pode usar, copiar, instalar em quantos computadores quiser e
distribuir.

**Precisa de internet?** Não. O programa não acessa a internet sozinho. As únicas exceções: se faltar o WebView2 no
computador, o instalador o baixa da Microsoft durante a instalação, e a versão portátil oferece abrir a página de
download, só se você clicar em "Sim".

**Funciona por cima de apresentação em tela cheia?** Funciona: a camada fica por cima de qualquer janela, inclusive
slides, vídeo e navegador em tela cheia. Para passar o slide, troque para **Mouse livre** (ou recolha a barra); para
voltar a escrever, **Ctrl+Alt+F** de qualquer programa.

**Funciona com a minha mesa digitalizadora?** Se ela funciona no Windows com o **Windows Ink** ligado no painel da
mesa, funciona no Giz na Tela, com pressão. Sem o Windows Ink ela desenha como mouse, com o traço sempre igual; o
próprio programa lembra disso no primeiro traço.

**E com projetor?** No modo "duplicar" funciona normalmente. No modo "estender", aparece na barra o botão de tela
(tecla Y): um toque leva a tinta e a barra para o projetor, outro traz de volta, e cada tela guarda a sua tinta.

**Como atualizar?** Na ajuda (F1), aba Sobre, **Procurar versão nova** diz se há versão nova e mostra as
novidades. **Instalar** baixa o instalador da página de versões, confere o SHA-256 publicado e instala por cima; o
programa fecha e abre de novo sozinho, e os ajustes e as imagens ficam. Nada é baixado nem instalado sem você clicar.
Quem tem a 1.2.1 ou anterior instala a versão nova à mão uma vez (pelo link acima); dali em diante, pelo Sobre. Na
versão portátil, baixe o zip novo.

**Posso perder a aula sem querer?** O **Esc** nunca fecha nem apaga: volta um passo de cada vez. O X de fechar, com
desenho na tela, pede um segundo toque. **Ctrl+Z** desfaz, inclusive o "Limpar". Para guardar, use **Salvar** (imagem)
ou **Mandar ao Giz Livre**.

**Onde ficam as imagens e os ajustes?** As imagens salvas, em `Documentos\Giz na Tela\capturas`. Os ajustes (onde a
barra fica, agentes de IA), em `%APPDATA%\Giz na Tela`. Na versão portátil, os ajustes ficam na pasta `dados`, ao lado
do programa.

**Como atualizo?** Baixe o instalador da versão nova e instale por cima: ele fecha o Giz na Tela, se estiver aberto, e
mantém os ajustes e as imagens.

**Os agentes de IA vêm ligados?** Não. Só funcionam se você ligar em Ajuda → Agentes de IA, e só para programas do
próprio computador.

**O antivírus reclamou. É seguro?** O programa passa pelo Microsoft Defender antes de cada publicação, e você pode
conferir o SHA-256 e a assinatura Sigstore de cada arquivo. O aviso de "editor desconhecido" aparece porque não há
assinatura digital paga. Veja [SECURITY.md](.github/SECURITY.md).

**Posso usar na minha escola ou rede de ensino?** Pode, à vontade, mantendo o aviso de licença.

**Encontrei um erro ou tenho uma ideia.** Abra uma [issue](https://github.com/HeraldoAntunes/giz-na-tela/issues/new/choose).

## Compilar a partir do código

Rust (rustup) e as ferramentas de compilação C++ do Visual Studio. Na pasta do projeto:

```powershell
cargo build --release
python tests\testar_versao.py
python tests\testar_comandos.py
```

O programa sai em `target\release\GizNaTela.exe`. O teste de comandos abre o programa com pastas temporárias, cobre a
tela por alguns segundos e confere cada comando, a segurança da API, os níveis dos agentes e o MCP. O instalador e a
versão portátil saem de `installer\build.ps1` (Inno Setup 6).

## Privacidade

O Giz na Tela não manda nada para a internet por conta própria. Ele só se conecta quando você clica em **Procurar
versão nova**, na ajuda: aí consulta a página do programa no GitHub. Se você mandar instalar, ele baixa o instalador de
lá e confere se o arquivo é o original antes de abrir.

As imagens e os ajustes ficam só no seu computador. "Mandar ao Giz Livre" entrega a imagem ao Giz Livre aberto no
mesmo computador, sem passar pela internet. Os agentes de IA vêm desligados; se você ligar, só programas do próprio
computador conseguem usar, e com um código de acesso que o Giz na Tela cria e guarda.

## Contribuir

Sugestões e correções são bem-vindas. Veja [CONTRIBUTING.md](.github/CONTRIBUTING.md) e a política de segurança em
[SECURITY.md](.github/SECURITY.md).

## Como citar

Se usar o Giz na Tela num trabalho, artigo ou material didático, cite a versão usada. Os dados estão em
[CITATION.cff](CITATION.cff); no GitHub, o botão **Cite this repository** gera a referência pronta (APA ou BibTeX).
Cada versão fica arquivada no Zenodo com DOI. O DOI geral, que leva sempre à versão mais recente, é
[10.5281/zenodo.23690523](https://doi.org/10.5281/zenodo.23690523).

## Como foi feito

Desenvolvido por Heraldo Antunes com apoio de assistentes de programação de inteligência artificial, usados como
ferramenta: a concepção, as decisões, a revisão e os testes são do autor, que responde pelo programa. O código é todo
próprio; ideias de recursos vieram do uso de outros anotadores de tela em sala de aula.

## Licença

MIT, © 2026 Heraldo Antunes. As bibliotecas usadas e as licenças delas estão em
[THIRD-PARTY-NOTICES.md](THIRD-PARTY-NOTICES.md).

## Programas Livres <img src="docs/img/bandeira-brasil.svg" alt="Brasil" title="Feito no Brasil" width="20" height="14">

Programas gratuitos e de código aberto, feitos por um professor para as próprias aulas. Cada um funciona sozinho,
sem internet e sem conta.

| | Programa | Para que serve | Código | Baixar |
|---|---|---|---|---|
| <img src="https://raw.githubusercontent.com/HeraldoAntunes/giz-livre/main/docs/img/icone.png" alt="" width="36"> | **Giz Livre** | Lousa digital para dar aula com mesa digitalizadora, caneta, toque ou mouse | [GitHub](https://github.com/HeraldoAntunes/giz-livre) | [Baixar](https://github.com/HeraldoAntunes/giz-livre/releases/latest) |
| <img src="https://raw.githubusercontent.com/HeraldoAntunes/giz-na-tela/main/docs/img/icone.png" alt="" width="36"> | **Giz na Tela** | **Escrever por cima de qualquer programa aberto no computador · você está aqui** | [GitHub](https://github.com/HeraldoAntunes/giz-na-tela) | [Baixar](https://github.com/HeraldoAntunes/giz-na-tela/releases/latest) |
| <img src="https://raw.githubusercontent.com/HeraldoAntunes/bancada-livre/main/src/assets/icone.svg" alt="" width="36"> | **Bancada Livre** | Simulador de lógica digital: portas, CIs 74xx, flip-flops e tabela-verdade | [GitHub](https://github.com/HeraldoAntunes/bancada-livre) | [Baixar](https://github.com/HeraldoAntunes/bancada-livre/releases/latest) |
| <img src="https://raw.githubusercontent.com/HeraldoAntunes/tarja-livre/main/docs/img/icone.svg" alt="" width="36"> | **Tarja Livre** | Tarjar dados pessoais em PDF sem internet, de um jeito que a tarja não sai | [GitHub](https://github.com/HeraldoAntunes/tarja-livre) | [Baixar](https://github.com/HeraldoAntunes/tarja-livre/releases/latest) |

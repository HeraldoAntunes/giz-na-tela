# <img src="docs/img/icone.png" alt="" width="44" align="top"> Giz na Tela

[![Testes](https://github.com/HeraldoAntunes/giz-na-tela/actions/workflows/testes.yml/badge.svg)](https://github.com/HeraldoAntunes/giz-na-tela/actions/workflows/testes.yml)
[![OpenSSF Scorecard](https://api.scorecard.dev/projects/github.com/HeraldoAntunes/giz-na-tela/badge)](https://scorecard.dev/viewer/?uri=github.com/HeraldoAntunes/giz-na-tela)
[![Versão](https://img.shields.io/github/v/release/HeraldoAntunes/giz-na-tela?label=vers%C3%A3o)](https://github.com/HeraldoAntunes/giz-na-tela/releases/latest)
[![Downloads](https://img.shields.io/github/downloads/HeraldoAntunes/giz-na-tela/total?label=downloads)](https://github.com/HeraldoAntunes/giz-na-tela/releases)
[![DOI](https://img.shields.io/badge/DOI-10.5281%2Fzenodo.23690523-1682D4)](https://doi.org/10.5281/zenodo.23690523)
[![Licença MIT](https://img.shields.io/badge/licen%C3%A7a-MIT-blue)](LICENSE)
[![Testado no Windows 11](https://img.shields.io/badge/testado%20no-Windows%2011-0078D6)](#requisitos)

**Escrever por cima de qualquer programa: slides, PDF, navegador, vídeo, simulação.** Uma camada transparente cobre a
tela e uma barra pequena traz caneta, marca-texto, formas, texto, numeração, visto e X, holofote e lupa. Feito por um
professor para as próprias aulas com mesa digitalizadora; funciona também com mouse e tela de toque.

<p align="center"><img src="docs/img/uso-slide.png" alt="Slide de física anotado com o Giz na Tela: título destacado com marca-texto, itens numerados, fórmula circulada com visto, seta até o bloco e texto explicativo; a barra de ferramentas fica à direita" width="860"></p>

### ⬇ Qual arquivo baixar? (versão 1.0.1)

| Para quê | Baixar |
|---|---|
| **Instalar no computador (Windows)** | [**Instalar-GizNaTela-1.0.1.exe**](https://github.com/HeraldoAntunes/giz-na-tela/releases/download/v1.0.1/Instalar-GizNaTela-1.0.1.exe) |
| Usar sem instalar (versão portátil, para pendrive) | [GizNaTela-1.0.1-portatil.zip](https://github.com/HeraldoAntunes/giz-na-tela/releases/download/v1.0.1/GizNaTela-1.0.1-portatil.zip) |

Se o Windows avisar "o Windows protegeu o computador", clique em **Mais informações → Executar assim mesmo** (o
programa é gratuito e não tem assinatura paga). A conferência dos arquivos fica na
[página de versões](https://github.com/HeraldoAntunes/giz-na-tela/releases).

- **Grátis e livre:** licença [MIT](LICENSE). Use, copie e distribua à vontade, inclusive em escolas e redes de ensino.
- **Sem internet, sem conta, sem anúncios:** nada sai do computador.
- **Pequeno:** um programa só, de cerca de 3,5 MB, sem nada a mais para instalar no Windows 11.

> **Aviso:** projeto independente e voluntário, fornecido "no estado em que se encontra", **sem garantia de qualquer
> tipo**. As marcas citadas pertencem aos seus donos; o Giz na Tela é um projeto independente, sem vínculo com elas.

## Como se usa

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
- **Holofote e lupa.**
- **Congelar a tela** (um vídeo parado enquanto você explica) e **ampliar** um ponto para escrever em cima.
- **Tinta que some** sozinha em 3 segundos, para apontar sem sujar a tela.
- **Salvar** a tela com a tinta como imagem (`Documentos\Giz na Tela\capturas`) ou **mandar ao
  [Giz Livre](https://github.com/HeraldoAntunes/giz-livre)** como quadro novo (abre o Giz se estiver fechado).
- **Fechar com desenho na tela** pede um segundo toque no X, para não perder a aula sem querer.
- Seis cores, três espessuras e atalhos de teclado para tudo (lista completa na ajuda).

## Comandos para tudo: atalhos, testes e agentes de IA

Cada ação da barra é um comando com nome, descrição e parâmetros (`web/comandos.js`). O botão, o atalho de teclado,
o teste automático e o agente de IA usam o mesmo caminho, então o que um faz o outro também faz.

Para assistentes de IA compatíveis com MCP, o próprio programa é o servidor (`GizNaTela.exe --mcp`), falando com uma
API local que só aceita programas do mesmo computador, com senha. **Vem desligado.** Na ajuda (aba Agentes de IA) o
professor escolhe:

- **Só ler:** o agente vê o estado e um retrato da tela com as anotações;
- **Agir:** o agente também escreve, desenha, aponta e congela a tela. Cada ação aparece com o aviso "Agente: ..." e
  sai com um Ctrl+Z.

A mesma aba mostra o trecho pronto para a configuração do assistente. Detalhes de segurança no
[SECURITY.md](.github/SECURITY.md).

## Requisitos

- Windows 10 ou 11, 64 bits. Não precisa de administrador (instala só para a sua conta, ou para todos, se quiser).
- O WebView2 da Microsoft, que desenha a barra e a tinta. Já vem no Windows 11 e nos Windows 10 atualizados; se
  faltar, o instalador o instala junto (precisa de internet nessa hora) e a versão portátil explica onde baixar.
- Mesa digitalizadora: para a pressão funcionar, ligue **"Usar Windows Ink"** no painel de ajustes da mesa. Testado no
  Windows 11 com mesa digitalizadora de caneta.

Limitação conhecida: a camada cobre só o monitor principal. Com projetor no modo "estender", escreva na tela principal
(ou use o modo "duplicar").

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

## Como foi feito

Com apoio de assistentes de programação de inteligência artificial, usados como ferramenta: a concepção, as decisões,
a revisão e os testes são do autor. O código é todo próprio; ideias de recursos vieram do uso de outros anotadores de
tela em sala de aula. Como contribuir: [CONTRIBUTING.md](.github/CONTRIBUTING.md).

## Como citar

Antunes, H. *Giz na Tela* (software livre). Zenodo. <https://doi.org/10.5281/zenodo.23690523>. Esse DOI aponta sempre
para a versão mais recente; o botão "Cite this repository", no alto da página do GitHub, dá a citação pronta.

## Licença

MIT, © 2026 Heraldo Antunes. As bibliotecas usadas e as licenças delas estão em
[THIRD-PARTY-NOTICES.md](THIRD-PARTY-NOTICES.md).

# Mudanças do Giz na Tela

Numeração: só correções muda o terceiro número (1.0.0 → 1.0.1); recurso novo muda o segundo (1.0.x → 1.1.0).

## [1.1.1] - 2026-10-27

Sem mudança no uso.

- **Retrato da tela com código próprio do Windows** (Congelar, Lupa, Ampliar, Salvar, Mandar ao Giz e o retrato pedido
  pelos agentes): sai uma biblioteca grande de captura e, com ela, os avisos de segurança das partes dela que nem
  entravam no programa. O programa fica menor por dentro.
- `osv-scanner.toml` explica os três avisos que sobram: são da parte Linux do Tauri ou só de compilação e não entram
  no programa do Windows.

## [1.1.0] - 2026-10-27

- **Setas mais nítidas:** a ponta vem aberta, em V, grande o bastante para ler de longe, e a haste ficou um quarto
  mais fina que o traço escolhido.
- **Seta dupla:** cada toque a mais no botão da seta (ou S de novo) passa pelos tipos: simples com ponta aberta,
  simples com ponta cheia, dupla com ponta aberta e dupla com ponta cheia. O ícone do botão mostra o tipo.
- **Cota** (botão novo, tecla K): arraste de uma ponta à outra do que quer medir; ao soltar, digite a medida (ou só
  Enter para deixar sem), que entra no meio da linha, sempre de pé para ler. O segundo toque troca as setas pelos
  tracinhos de arquitetura.
- **Medir** (botão da régua, tecla J): arraste e a medida aparece sozinha, em pixels da tela. Uma Cota com a medida
  digitada com unidade (ex.: 10 cm) sobre algo de tamanho conhecido vira a régua, e o Medir passa a dar cm.
- **Mover** (a mão, tecla Espaço): toque num traço, forma, texto ou carimbo e arraste; o Ctrl+Z devolve ao lugar.
- **Verde como cor de início**, na primeira bolinha (tecla 1). Tocar de novo na bolinha escolhida troca aquela cor pelo
  seletor de cores do Windows; a ajuda tem "Voltar às cores de fábrica".
- **Bandeira do Brasil** ao lado do nome, no topo da ajuda (Sobre).
- **O programa lembra as escolhas:** cor, espessura, tipos de seta e de cota, preenchimento e as cores das bolinhas
  voltam como estavam ao abrir de novo.

## [1.0.1] - 2026-10-27

Versão pequena, sem mudança no uso.

- **Ajuda → Sobre** mostra o endereço do código-fonte e das versões novas (github.com/HeraldoAntunes/giz-na-tela). O
  programa continua sem consultar a internet sozinho.
- Publicada também para o arquivo permanente do código no Zenodo, que dá ao Giz na Tela um identificador para
  citação (DOI).

## [1.0.0] - 2026-10-26

Primeira versão. Uma camada transparente por cima de qualquer programa, para escrever com a mesa digitalizadora, o
mouse ou a tela de toque.

- **Ferramentas:** caneta com pressão, marca-texto, borracha, reta, seta, retângulo, elipse (com preenchimento
  translúcido ou cheio), texto, numeração, visto e X, holofote e lupa; seis cores e três espessuras.
- **Tela:** quadro branco ou negro por cima de tudo, congelar a tela, ampliar um ponto para escrever em cima, tinta
  que some sozinha em 3 segundos.
- **Modo em palavras:** "Escrevendo" ou "Mouse livre" (o clique vai para os outros programas e a tinta fica);
  Ctrl+Alt+F volta a escrever de qualquer programa. Esc volta um passo e nunca fecha.
- **Barra recolhível** num botão verde que se arrasta; o programa lembra a posição e se estava recolhida.
- **Fechar com desenho na tela** pede um segundo toque no X.
- **Guardar:** salvar a tela com a tinta como imagem; mandar ao Giz Livre como quadro novo (abre o Giz se estiver
  fechado).
- **Dica do Windows Ink** quando a mesa chega como mouse (sem pressão), no máximo três vezes.
- **Comandos para tudo:** cada ação da barra é um comando nomeado, o mesmo para botão, atalho, agente de IA e teste.
- **Agentes de IA (opcional, desligado de fábrica):** API local com senha e servidor MCP (`GizNaTela.exe --mcp`), nos
  níveis "só ler" e "agir"; cada ação de agente mostra "Agente: ..." e sai com um Ctrl+Z.
- **Ajuda em português** (F1 ou o "?" da barra), com os atalhos tirados do próprio registro de comandos.

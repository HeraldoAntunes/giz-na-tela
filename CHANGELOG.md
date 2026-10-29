# Mudanças do Giz na Tela

Numeração: só correções muda o terceiro número (1.0.0 → 1.0.1); recurso novo muda o segundo (1.0.x → 1.1.0).

## [1.3.2] - 2026-10-28

Delete que limpa a tela.

- **Delete sem nada escolhido limpa a tela:** como no Giz Livre. O primeiro Delete avisa, o segundo (em até 3
  segundos) apaga toda a tinta; Ctrl+Z desfaz. Com o laço, o Delete continua apagando só o grupo.

## [1.3.1] - 2026-10-28

Fixar na barra de tarefas e texto de privacidade mais claro.

- **Fixar na barra de tarefas:** a barra de ferramentas ganhou botão próprio na barra de tarefas, com o ícone e o mesmo
  ID dos atalhos do instalador; dá para fixar o Giz na Tela ali. A camada de desenho continua sem botão, e tocar no
  botão não esconde a barra.
- **Privacidade:** o texto do README e da ajuda agora diz que, ao mandar instalar, o programa baixa o instalador e
  confere o arquivo, e que os agentes usam um código de acesso criado pelo programa (não uma senha).

## [1.3.0] - 2026-10-28

Atualizar pelo próprio programa e correções da revisão de código.

- **Atualizar pelo próprio programa:** na ajuda (Sobre), **Procurar versão nova** consulta o GitHub só quando você
  clica e diz se há versão nova, com as novidades. **Instalar** (com um segundo toque para confirmar) baixa o
  instalador da página de versões, confere o SHA-256 publicado junto e instala por cima; o Giz na Tela fecha e abre de
  novo sozinho. O instalador anota o que fez em `Documentos\Giz na Tela\atualizacao.log`. A versão portátil só mostra
  o endereço para baixar. Quem tem a 1.2.1 ou anterior instala esta versão à mão uma vez; dali em diante, pelo Sobre.
- Se o programa parar de repente, a causa fica anotada no arquivo `erros.txt`, na pasta de ajustes dele.
- **Alt+F4** na tela ou na barra agora passa pelo X: com desenho na tela, pede o segundo toque. Antes fechava sem
  perguntar (e a tinta se perdia) ou fechava só a barra, deixando a tela coberta sem controles.
- Um agente de IA não fecha mais o Giz na Tela com desenho na tela.
- Os ajustes ganham uma cópia de segurança: se o arquivo estragar, o programa usa a cópia em vez de voltar tudo ao
  de fábrica.
- Leitores de tela: os botões dizem se estão ligados, as bolinhas dizem o nome da cor, e as abas da ajuda são
  anunciadas como abas.
- Textos corrigidos (a dica do Salvar citava a pasta de outro programa).

## [1.2.1] - 2026-10-27

Versão pequena, só a correção abaixo.

- **Correção importante:** em tela baixa (notebook de 768 px com zoom de 125%), a barra nova é mais alta que a tela e
  o programa podia fechar sozinho ao abrir a barra recolhida ou ao abrir de novo. Agora a barra encolhe para caber e
  rola, sem esconder Desfazer, Salvar e o X.

## [1.2.0] - 2026-10-27

Pacote de melhorias. Quem já tem a 1.1.x ou a 1.0.x pode instalar por cima: os ajustes e as imagens ficam.

- **Projetor e outras telas:** com o projetor no modo estender, o botão de tela (tecla Y) leva a tinta e a barra para
  a outra tela, e traz de volta. Cada tela guarda a sua tinta. O programa abre na tela da última vez, se ela ainda
  estiver ligada. Salvar, copiar, congelar e a lupa usam a tela onde a tinta está.
- **Canetas prontas:** três bolinhas embaixo das espessuras, como três canetas na mesa: cada uma tem a sua cor, a
  sua espessura e é caneta ou marca-texto. Um toque (ou F2, F3 e F4) pega a caneta; com ela na mão, mudar a cor ou a
  espessura muda a própria caneta. "Voltar às cores de fábrica", na ajuda, volta também as canetas.
- **Laço** (tecla W, botão novo ao lado da mão; o Congelar passou para a fileira do quadro): contorne vários
  traços, formas e textos; depois arraste o grupo, toque numa cor para pintá-lo ou aperte Delete para apagar. Esc solta o grupo.
- **Copiar a tela** (Ctrl+C, botão novo): a tela com a tinta vai para a área de transferência, para colar num
  documento ou numa prova.
- **Botão lateral da caneta** (experimental, na ajuda, aba Mesa digitalizadora): enquanto apertado, vale a borracha,
  o marca-texto, a reta, a mão ou o laço; ao soltar, volta a ferramenta de antes. De fábrica, desligado. Com o
  mouse, vale o botão direito.
- **Shift na caneta e no marca-texto** deixa o traço reto (de 15 em 15 graus), como na reta.
- A ajuda (Sobre) dizia que o retrato da tela usava uma biblioteca que saiu na 1.1.1; agora diz o certo.

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

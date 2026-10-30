# Segurança e privacidade

## Como o Giz na Tela protege o computador e os dados
- **Funciona offline.** Não tem telemetria, contas ou anúncios, e o programa não envia nada seu para a internet. Ele
  só se conecta em dois casos, sempre por pedido do usuário:
  - **Procurar versão nova** (ajuda, aba Sobre): consulta a página de versões no GitHub, por HTTPS e só nos endereços
    do GitHub. Instalar só acontece depois de confirmar, com o arquivo conferido pelo SHA-256 publicado.
  - **Sem o WebView2:** pode abrir a página oficial do WebView2 da Microsoft, se o usuário clicar em "Sim".
- **O componente WebView2 da Microsoft**, que desenha a janela, pode abrir sozinho uma conexão com a Microsoft. O
  programa desliga os serviços de rede dele e evita a conexão que ele abria ao reabrir, mas só em parte das aberturas:
  nas medições, 2 de 5 aberturas de 10 minutos ficaram sem conexão. Nada seu passa por ela, e o firewall do Windows pode bloqueá-la.
- **Retratos da tela ficam no computador.** "Salvar" grava a imagem em `Documentos\Giz na Tela\capturas`; "Mandar ao
  Giz Livre" entrega a imagem ao Giz Livre aberto no mesmo computador (endereço local `127.0.0.1`).
- **Agentes de IA: desligados de fábrica.** A API local só existe para programas do próprio computador e:
  - escuta apenas em `127.0.0.1`, inacessível pela rede;
  - sem "Agentes de IA" ligado na ajuda, não há token e todo pedido é recusado;
  - exige o token (cabeçalho `X-Giz-Tela`), gerado pelo gerador aleatório do Windows e guardado só na pasta de dados
    da conta (`%APPDATA%\Giz na Tela\agente`); desligar e ligar de novo troca o token;
  - recusa `Host` diferente de `127.0.0.1`/`localhost` na porta dela (bloqueia *DNS rebinding*) e qualquer pedido com
    `Origin` (página aberta no navegador);
  - limita o pedido a 1 MB e valida cada parâmetro de cada comando (tipo, faixa, valores aceitos, parâmetro a mais);
  - tem dois níveis escolhidos pelo professor: **só ler** (estado e retrato da tela) e **agir** (desenhar e mudar a
    tela). Toda ação de agente aparece na tela ("Agente: ...") e sai com um Ctrl+Z.
- **Valor de fora nunca vira código.** O que chega pela API passa para as páginas só serializado em JSON; a camada
  só aceita uma lista fixa de funções.
- **Executável conferível.**
  - Cada versão vem com `SHA256SUMS.txt`.
  - Cada anexo da release vem com uma assinatura do Sigstore (`<arquivo>.sigstore.json`), feita pelo workflow
    `assinar-release.yml` deste repositório. Para conferir (com o
    [cosign](https://docs.sigstore.dev/cosign/system_config/installation/)):
    `cosign verify-blob --bundle Instalar-GizNaTela-<versão>.zip.sigstore.json --certificate-identity-regexp "^https://github.com/HeraldoAntunes/giz-na-tela/.github/workflows/assinar-release.yml@" --certificate-oidc-issuer https://token.actions.githubusercontent.com Instalar-GizNaTela-<versão>.zip`
  - O programa é compilado sem compactadores, com as dependências travadas no `Cargo.lock` (`--locked`), e verificado
    no Microsoft Defender antes da publicação (sem o Defender, o pacote não sai). O pacote "escola sem internet" só
    leva o instalador oficial do WebView2 se a assinatura digital dele for válida e da Microsoft, e o programa confere
    de novo antes de rodá-lo.
  - O SHA-256 que o "Procurar versão nova" confere vem da mesma página de versões: ele prova que o arquivo chegou
    inteiro, não que a conta do GitHub esteja segura. Quem quiser essa garantia confere a assinatura do Sigstore.

## Aviso sobre antivírus
Programas sem assinatura digital paga podem receber o aviso "editor desconhecido" do SmartScreen. Antes de instalar,
compare o SHA-256 do arquivo baixado com o publicado. Se preferir, compile a partir do código-fonte (veja o README).

## Versões com suporte
Só a versão mais recente recebe correções de segurança. A versão instalada aparece no topo da ajuda (tecla F1).

## Como relatar uma vulnerabilidade
Não abra uma *issue* pública com os detalhes da falha. Use o aviso privado do GitHub:
<https://github.com/HeraldoAntunes/giz-na-tela/security/advisories/new>
(no repositório, **Security → Report a vulnerability**). Informe a versão, o passo a passo para reproduzir e o impacto.

O que esperar:
- confirmação de recebimento em até 7 dias;
- avaliação e plano de correção em até 30 dias;
- a correção sai numa versão nova, com a falha descrita no CHANGELOG e no aviso de segurança publicado depois da
  correção (divulgação coordenada). Quem relatou recebe crédito, se quiser.

Correções de segurança têm prioridade sobre qualquer outro trabalho no projeto.

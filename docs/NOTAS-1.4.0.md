<p><img src="https://raw.githubusercontent.com/HeraldoAntunes/giz-na-tela/v1.4.0/docs/img/icone.png" alt="" width="64"></p>

## ⬇ Qual arquivo baixar?

| Para quê | Arquivo | |
|---|---|---|
| **Instalar no computador (Windows)** | Instalar-GizNaTela-1.4.0.zip | [**Baixar**](https://github.com/HeraldoAntunes/giz-na-tela/releases/download/v1.4.0/Instalar-GizNaTela-1.4.0.zip) |
| Escola sem internet (leva o WebView2 da Microsoft junto) | GizNaTela-1.4.0-sem-internet.zip | [Baixar](https://github.com/HeraldoAntunes/giz-na-tela/releases/download/v1.4.0/GizNaTela-1.4.0-sem-internet.zip) |
| Usar sem instalar (versão portátil, para pendrive) | GizNaTela-1.4.0-portatil.zip | [Baixar](https://github.com/HeraldoAntunes/giz-na-tela/releases/download/v1.4.0/GizNaTela-1.4.0-portatil.zip) |

Abra o zip e dê dois cliques em **Instalar-GizNaTela-1.4.0.exe**: ele pergunta se instala na sua conta (sem pedir
administrador) ou se só abre, sem instalar. Os outros arquivos da lista (SHA256SUMS e .sigstore.json) servem só para
conferir o download. Se o Windows avisar "o Windows protegeu o computador", clique em **Mais informações → Executar
assim mesmo** (o programa é gratuito e não tem assinatura paga).

---

## O que mudou
ATENÇÃO, QUEM TEM A 1.3.x: o botão Instalar desta tela vai dizer "a release não tem o instalador". Isso é esperado e
não é defeito: desta vez o instalador vem dentro de um zip. Baixe o Instalar-GizNaTela-1.4.0.zip na página de versões
do Giz na Tela no GitHub, abra o zip e dê dois cliques no instalador: ele instala por cima, e os ajustes e as imagens
salvas ficam. Da 1.4.0 em diante, a atualização por esta tela volta a funcionar sozinha.

- **Instalador novo, sem Inno Setup:** o próprio programa se instala, na sua conta e sem administrador. Ele tira a
  versão anterior, e os ajustes, as imagens salvas e o botão fixado na barra de tarefas ficam. Se houver um Giz na
  Tela aberto com desenho, ele pergunta antes de fechar.
- **Escola sem internet:** um zip com o instalador e o WebView2 da Microsoft ao lado, para o Windows 10 que não o tem.
- **Correções de uma auditoria independente:**
  - fechar com desenho em outra tela, ou com um texto ainda sendo digitado, pede confirmação;
  - a Mão não apaga o Refazer;
  - um toque deixa um ponto;
  - um segundo dedo não emenda o traço;
  - a escrita bem leve sai fina.
- **Menos conexões do WebView2 com a Microsoft:** o componente que desenha a janela abria uma conexão sozinho ao reabrir;
  o programa agora evita isso em parte das aberturas (nas medições, 2 de 5 aberturas de 10 minutos sem conexão).
- **Mais firme ao abrir:** se o WebView2 falhar ao criar a janela, o programa tenta de novo e, se não der, explica o que
  fazer.

**Como foi testada:** passou por uma auditoria independente (aprovada) e pelos testes curtos no Windows (comandos,
quedas e reabertura, instalação por cima da 1.3.2, migração e PC limpo), e os três arquivos deram zero alertas em 75
antivírus no VirusTotal. Os testes longos desta montagem (30 minutos aberto e a medição de conexões) e os testes à mão
com a mesa digitalizadora e o projetor ficam para depois desta publicação.

**Conferir o download:** o `SHA256SUMS.txt` traz o SHA-256 de cada arquivo (no Windows: `certutil -hashfile Instalar-GizNaTela-1.4.0.zip SHA256`). Cada arquivo também tem uma assinatura digital gratuita do Sigstore (o `.sigstore.json` com o mesmo nome), feita pelo GitHub Actions deste repositório depois de publicado: ela prova que o arquivo baixado é o mesmo que foi assinado ali, não como ele foi montado. Como conferir: [SECURITY.md](https://github.com/HeraldoAntunes/giz-na-tela/blob/main/.github/SECURITY.md).

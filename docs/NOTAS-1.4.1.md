<p><img src="https://raw.githubusercontent.com/HeraldoAntunes/giz-na-tela/main/docs/img/icone.png" alt="" width="64"></p>

## ⬇ Qual arquivo baixar?

| Para quê | Arquivo | |
|---|---|---|
| **Instalar no computador (Windows)** | Instalar-GizNaTela-1.4.1.zip | [**Baixar**](https://github.com/HeraldoAntunes/giz-na-tela/releases/download/v1.4.1/Instalar-GizNaTela-1.4.1.zip) |
| Escola sem internet (leva o WebView2 da Microsoft junto) | GizNaTela-1.4.1-sem-internet.zip | [Baixar](https://github.com/HeraldoAntunes/giz-na-tela/releases/download/v1.4.1/GizNaTela-1.4.1-sem-internet.zip) |
| Usar sem instalar (versão portátil, para pendrive) | GizNaTela-1.4.1-portatil.zip | [Baixar](https://github.com/HeraldoAntunes/giz-na-tela/releases/download/v1.4.1/GizNaTela-1.4.1-portatil.zip) |

Quem tem a 1.4.0 pode atualizar pelo próprio programa: **Ajuda e ajustes → Procurar versão nova → Instalar**. Ou abra
o zip e dê dois cliques em **Instalar-GizNaTela-1.4.1.exe**: ele instala por cima, e os ajustes e as imagens salvas
ficam. Quem tem a 1.3.x precisa baixar o zip aqui uma vez (o botão Instalar da 1.3.x não acha o instalador dentro do
zip). Os outros arquivos da lista (SHA256SUMS e .sigstore.json) servem só para conferir o download. Se o Windows avisar
"o Windows protegeu o computador", clique em **Mais informações → Executar assim mesmo** (o programa é gratuito e não
tem assinatura paga).

---

## O que mudou

Versão de correções.

- **Delete mantido pressionado não limpa a tela:** sem nada escolhido pelo laço, o Delete limpa a tela inteira só com
  um segundo toque, para confirmar. Manter a tecla apertada contava como o segundo toque. Agora só um segundo toque de
  verdade limpa.
- **Com duas telas, a confirmação de limpar fica na mesma tela:** apertar Delete numa tela, passar à outra e apertar
  Delete de novo limpava a outra tela. Trocar de tela agora desarma a confirmação.
- **Menos conexão do WebView2 com a Microsoft:** o componente que desenha a janela abria sozinho uma conexão com a
  Microsoft (nas medições da 1.4.0, em 5 de 5 aberturas de 10 minutos). Além de desligar os serviços de rede dele, o
  programa agora o impede de achar os endereços da internet. O "Procurar versão nova" não passa pelo componente e
  continua funcionando. Nada seu passa por essa conexão, e o firewall do Windows pode bloqueá-la.

Os dois primeiros achados vieram de uma revisão independente do código; em ambos, Ctrl+Z já trazia a tinta de volta.

**Como foi testada:** os testes automáticos sem janela (os do Rust e os do teclado, que reproduzem os dois casos do
Delete), a porta de publicação (os três arquivos deram zero alertas em 75 antivírus no VirusTotal) e, numa máquina
virtual Windows 11 de testes, os testes com janela: comandos, quedas e reabertura, camada, falhas do WebView2,
instalação e desinstalação, PC limpo, sem internet (mensagem clara no Procurar versão nova, nenhuma conexão) e a
medição de conexões do programa instalado: **5 de 5 aberturas de 10 minutos sem nenhuma conexão para fora** (na 1.4.0,
5 de 5 tiveram uma). A máquina virtual não substitui a mesa digitalizadora, o projetor e duas telas de verdade, que
ficam para o uso em aula.

**Conferir o download:** o `SHA256SUMS.txt` traz o SHA-256 de cada arquivo (no Windows: `certutil -hashfile Instalar-GizNaTela-1.4.1.zip SHA256`). Cada arquivo também tem uma assinatura digital gratuita do Sigstore (o `.sigstore.json` com o mesmo nome), feita pelo GitHub Actions deste repositório depois de publicado: ela prova que o arquivo baixado é o mesmo que foi assinado ali, não como ele foi montado. Como conferir: [SECURITY.md](https://github.com/HeraldoAntunes/giz-na-tela/blob/main/.github/SECURITY.md).

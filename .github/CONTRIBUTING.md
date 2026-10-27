# Como contribuir

Obrigado pelo interesse! O Giz na Tela é software livre (MIT), feito para professores.

- **Relatar um problema:** abra uma *issue* com a versão, o Windows, a mesa digitalizadora (se houver), o passo a passo e,
  se possível, um print. Problemas de segurança vão pelo caminho descrito em [SECURITY.md](SECURITY.md).
- **Propor uma melhoria:** descreva a situação de aula que ela resolve.
- **Enviar código:**
  1. **Cada ação é um comando.** Recurso novo entra primeiro no registro (`web/comandos.js`: nome, descrição, nível e
     parâmetros), depois na execução da barra (`EXEC` em `web/barra.html`) e ganha um teste em
     `tests/testar_comandos.py`. O teste falha se faltar algum dos três. Botão e atalho chamam o comando; nada de
     lógica só no clique.
  2. Sem dependências novas sem necessidade: Rust só com tauri, serde, serde_json, png, base64 e o atalho global;
     JavaScript puro nas páginas; testes só com a biblioteca padrão do Python.
  3. Valor que vem de fora (API, MCP) entra no JavaScript só serializado em JSON, nunca colado como código.
  4. Antes do *pull request*, rode `cargo build --release`, `python tests/testar_versao.py` e
     `python tests/testar_comandos.py` (todos devem passar). O teste de comandos usa pastas temporárias e nunca toca
     nos seus ajustes nem nas suas imagens.
  5. Textos da interface em português do Brasil, com acentuação correta.

Ao contribuir, você concorda em licenciar a sua contribuição sob a licença MIT do projeto e em seguir o
[Código de Conduta](CODE_OF_CONDUCT.md).

## Uso de IA generativa

Assistentes de programação de IA podem ser usados como ferramenta, inclusive pelo autor do projeto. A regra segue a
prática de projetos como o kernel do Linux:

- **A responsabilidade é de quem envia.** A pessoa revisa e entende todo o código, confere que nada foi copiado de
  fonte com licença incompatível com a MIT e responde pela contribuição. A IA não é autora nem coautora.
- **Declare o uso no commit**, com a linha final `Assisted-by: <ferramenta>:<modelo>`, por exemplo
  `Assisted-by: Claude:claude-opus-5-5`. Não use `Co-authored-by` para ferramentas: ele é reservado a pessoas.
- **Diga no *pull request*** o que a ferramenta fez e o que foi verificado (testes rodados, comportamento conferido).
  Relato de falha encontrado com ajuda de IA precisa de passo a passo que reproduza o problema.

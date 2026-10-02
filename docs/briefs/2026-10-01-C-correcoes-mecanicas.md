# Brief — Aplicar correções mecânicas da revisão cruzada

- **Data**: 2026-10-01 (turno da noite) · **Executor**: subagente Codex · **Status**: pendente

## Tarefa

Ler `docs/sdd/REVISAO-CRUZADA.md`. Aplicar nos arquivos `docs/sdd/0*.md` e `docs/sdd/1*.md` **somente**
as correções marcadas "precisa do usuário? não". Para cada correção aplicada, marcar na revisão a
coluna/linha como "aplicada em AAAA-MM-DD"; as marcadas "sim" ficam intocadas.

Restrições:
- Não altere o sentido de decisões propostas; só nomes, referências, unidades, numeração, termos e
  alinhamento com o que já está Decidido/Aceito.
- Não edite o GDD, os ADRs, o STATUS nem `docs/PERGUNTAS-ABERTAS.md`.

## Regras

- Português do Brasil, Markdown, UTF-8, LF. Leia arquivos sempre com encoding UTF-8.
- Não decida nada em nome do dono do projeto; decisões ficam como perguntas.
- Não rode git. Não instale nada. Arquivos permitidos: `docs/sdd/0*.md`, `docs/sdd/1*.md` e `docs/sdd/REVISAO-CRUZADA.md`.
- Ao terminar, responda com: arquivos alterados e um resumo de 5 linhas.

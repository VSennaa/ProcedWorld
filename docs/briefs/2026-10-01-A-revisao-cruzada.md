# Brief — Revisão cruzada do SDD

- **Data**: 2026-10-01 (turno da noite) · **Executor**: subagente Codex · **Status**: pendente

## Tarefa

Ler `docs/sdd/*.md` (15 rascunhos), `docs/gdd/12-variaveis-e-formulas.md`, `docs/adr/*.md` e as seções
"## Decidido" de `docs/gdd/0*.md` e `docs/gdd/1*.md`. Escrever **somente** `docs/sdd/REVISAO-CRUZADA.md` com:

1. Tabela de inconsistências: id, arquivos, o que diverge (cite trechos curtos), proposta de correção,
   **precisa do usuário? (sim/não)**. "Não" só para correções mecânicas: nomes de tipos/campos,
   referências cruzadas, unidades, numeração, termos do glossário, contradição com algo já Decidido/Aceito
   (nesse caso o Decidido/Aceito vence).
2. Contratos compartilhados que aparecem com nomes diferentes (ex.: comando, intenção, hash, snapshot,
   evento do Ledger) e o nome canônico proposto.
3. Lacunas: o que algum SDD supõe existir e nenhum define.
4. Violações de invariantes do CLAUDE.md §2 (determinismo, IA só propõe, segredos, texto de jogador).

Seja específico (arquivo + seção). Sem elogios, sem resumo dos documentos.

## Regras

- Português do Brasil, Markdown, UTF-8, LF. Leia arquivos sempre com encoding UTF-8.
- Não decida nada em nome do dono do projeto; decisões ficam como perguntas.
- Não rode git. Não instale nada. Arquivo permitido: apenas `docs/sdd/REVISAO-CRUZADA.md`.
- Ao terminar, responda com: arquivos alterados e um resumo de 5 linhas.

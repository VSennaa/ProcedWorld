# Brief — Perguntas abertas consolidadas

- **Data**: 2026-10-01 (turno da noite) · **Executor**: subagente Codex · **Status**: pendente

## Tarefa

Ler as seções "Perguntas abertas" de `docs/gdd/*.md` e `docs/sdd/*.md`, a seção "Perguntas abertas" de
`docs/STATUS.md` e os ADRs com status Proposto ou partes em aberto (`docs/adr/README.md`). Escrever **somente**
`docs/PERGUNTAS-ABERTAS.md`:

- Agrupar por tema (design de jogo, IA, técnica/stack, infra/segurança, produto).
- Deduplicar perguntas iguais vindas de arquivos diferentes (cite todas as origens).
- Para cada pergunta: origem(ns), por que importa (1 linha), opções (2–4), **Recomendação** e se
  **bloqueia** a aprovação do GDD, a do SDD, ou nenhuma.
- No topo: as até 10 perguntas que bloqueiam a próxima fase, em ordem de prioridade.
- Não inclua perguntas já respondidas no "## Decidido" de algum pilar ou em ADR aceito.

## Regras

- Português do Brasil, Markdown, UTF-8, LF. Leia arquivos sempre com encoding UTF-8.
- Não decida nada em nome do dono do projeto; decisões ficam como perguntas.
- Não rode git. Não instale nada. Arquivo permitido: apenas `docs/PERGUNTAS-ABERTAS.md`.
- Ao terminar, responda com: arquivos alterados e um resumo de 5 linhas.

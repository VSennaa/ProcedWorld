# Brief — Estudo: Como outros jogos de estratégia funcionam

- **Data**: 2026-10-01 · **Executor**: subagente Codex **com busca na web** · **Status**: pendente

## Tarefa

Escrever **somente** `docs/research/como-outros-jogos-funcionam.md`: um estudo para orientar o design e a engenharia do
ProcedWorld (leia antes `CLAUDE.md` §1–3 e `docs/gdd/README.md`; consulte pontualmente os pilares citados).

Jogos: Civilization VI/VII (turnos simultâneos e dinâmicos, IA, diplomacia, eras), Old World
(personagens, legitimidade, eventos encadeados), Stellaris e Victoria 3 (pops, facções, crises de fim de
jogo), Humankind (eras e culturas), Polytopia (4X mobile em sessões curtas), Unciv (Civ V open source:
arquitetura, multiplayer por turnos assíncronos, mods/JSON de regras), Freeciv (servidor autoritativo).
Foco: turnos simultâneos e assíncronos, IA de oponentes e diplomacia coerente, economia sem inflação no
longo prazo, UX mobile de 4X, como regras ficam em dados.

## Formato

- Para cada jogo/projeto/tecnologia: o que é (2 linhas), **como funciona** o mecanismo relevante,
  o que **aproveitar** e o que **evitar** no ProcedWorld, e **qual pilar/SDD** isso afeta
  (`docs/gdd/NN-*.md`, `docs/sdd/NN-*.md`).
- Feche com "## Recomendações para o ProcedWorld" (5–10 itens acionáveis) e "## Fontes".
- **Toda afirmação factual precisa de fonte** (URL) na seção Fontes, preferindo documentação oficial,
  devlogs, wikis e papers. Se não achar fonte, diga "não verificado". Não invente números.
- Não copie trechos longos: resuma com suas palavras; no máximo uma citação curta por fonte.
- Português do Brasil, Markdown, 200–450 linhas, UTF-8, LF.

## Regras

- Arquivo permitido: apenas `docs/research/como-outros-jogos-funcionam.md`. Não rode git. Não instale nada.
- Grave só com a ferramenta de patch (apply_patch), nunca com PowerShell. Ao final rode
  `python tools/agents/check-encoding.py docs/research/como-outros-jogos-funcionam.md` e só termine com "encoding ok".

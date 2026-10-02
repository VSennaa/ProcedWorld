# Brief — Estudo: Jogos sandbox e mundos emergentes

- **Data**: 2026-10-01 · **Executor**: subagente Codex **com busca na web** · **Status**: pendente

## Tarefa

Escrever **somente** `docs/research/sandbox-e-mundos-emergentes.md`: um estudo para orientar o design e a engenharia do
ProcedWorld (leia antes `CLAUDE.md` §1–3 e `docs/gdd/README.md`; consulte pontualmente os pilares citados).

Jogos: Dwarf Fortress (geração de mundo e de história), Caves of Qud (história procedural de
sultões), RimWorld (storytellers: Cassandra, Phoebe, Randy), WorldBox (god sim mobile), Songs of Syx,
Crusader Kings III (personagens e dinastias), Ultima Ratio Regum, Kenshi. Foco: como sustentam jogo
infinito/sem vitória, como geram história e crônica a partir de estado, diretores de eventos e justiça,
colapso e renascimento, como o jogador entende causas.

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

- Arquivo permitido: apenas `docs/research/sandbox-e-mundos-emergentes.md`. Não rode git. Não instale nada.
- Grave só com a ferramenta de patch (apply_patch), nunca com PowerShell. Ao final rode
  `python tools/agents/check-encoding.py docs/research/sandbox-e-mundos-emergentes.md` e só termine com "encoding ok".

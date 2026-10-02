# Brief — Estudo: IA com LLM em jogos

- **Data**: 2026-10-01 · **Executor**: subagente Codex **com busca na web** · **Status**: pendente

## Tarefa

Escrever **somente** `docs/research/ia-com-llm-em-jogos.md`: um estudo para orientar o design e a engenharia do
ProcedWorld (leia antes `CLAUDE.md` §1–3 e `docs/gdd/README.md`; consulte pontualmente os pilares citados).

Projetos: CICERO (Meta, Diplomacy), Generative Agents (Stanford, Smallville), AI Dungeon, Suck Up!,
1001 Nights, Inworld/NPCs com LLM, Voyager (Minecraft), jogos de diplomacia com LLM e benchmarks de
agentes em jogos de estratégia. Foco: separar planejamento/decisão de linguagem (como CICERO), grounding,
memória e reflexão, custo e latência, prompt injection por jogadores e defesas, determinismo e replay,
o que deu errado em produção. Relacione com as camadas T0/T1/T2 e com Jev/Laya (modelos de decisão).

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

- Arquivo permitido: apenas `docs/research/ia-com-llm-em-jogos.md`. Não rode git. Não instale nada.
- Grave só com a ferramenta de patch (apply_patch), nunca com PowerShell. Ao final rode
  `python tools/agents/check-encoding.py docs/research/ia-com-llm-em-jogos.md` e só termine com "encoding ok".

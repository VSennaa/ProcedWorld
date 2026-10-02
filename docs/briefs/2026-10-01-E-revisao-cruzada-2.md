# Brief — Revisão cruzada 2 (SDDs 15–18) e correções mecânicas

- **Data**: 2026-10-01 (turno da noite) · **Executor**: subagente Codex · **Status**: pendente

## Tarefa

1. Ler `docs/sdd/REVISAO-CRUZADA.md` (nomes canônicos e decisões da primeira revisão), os SDDs novos
   `docs/sdd/15-regras-de-dominio.md`, `16-visibilidade.md`, `17-identidade-e-contas.md`,
   `18-dsl-catalogos.md`, e consultar pontualmente os demais SDDs, `docs/gdd/12-variaveis-e-formulas.md`
   e os ADRs.
2. Acrescentar ao final de `docs/sdd/REVISAO-CRUZADA.md` uma seção `## Revisão 2 (SDDs 15–18)` com a
   mesma tabela (id RC2-NN, arquivos, divergência, correção, precisa do usuário? sim/não).
3. Aplicar **somente** as correções "não" (nomes canônicos, referências, termos, alinhamento com o que
   já está Decidido/Aceito ou com a revisão 1) nos SDDs 15–18 e, se o conflito estiver em outro SDD, no
   próprio SDD de origem. Marcar cada uma como "aplicada em 2026-10-01".
4. Atualizar a seção "## Lacunas" da revisão: marcar como "coberta por SDD NN" as lacunas que os SDDs
   15–18 resolveram.

## Regras

- Português do Brasil, Markdown, UTF-8, LF. Não decida nada pelo dono do projeto.
- Arquivos permitidos: `docs/sdd/*.md`. Não edite GDD, ADRs, STATUS, PERGUNTAS-ABERTAS.
- Grave só com a ferramenta de patch (apply_patch), nunca com PowerShell. Ao final rode
  `python tools/agents/check-encoding.py` e só termine com "encoding ok".
- Não rode git. Não instale nada. Responda com arquivos alterados e um resumo de 5 linhas.

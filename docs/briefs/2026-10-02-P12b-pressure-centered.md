# Brief P12b — Pressão de crise com coesão centrada em 50

Decisão do usuário (2026-10-02, GDD 12 atualizado): na fórmula de `P_c`, trocar `− C / 5` por
`− (C − 50) / 5` (inteiros; arredondamento como o resto da fórmula). Atualizar a implementação em
`engine/crates/pw-engine` e os testes de unidade afetados, e **restaurar no teste de saúde do harness o
piso de pressão média ≥ 10** (faixa 10..=80), mantendo todos os outros limites (cidades ≥ 16,
população ≥ 40, colapsos ≤ 2, eventos da Entropia ≥ 10, auditoria diplomática). Se, mesmo assim, o piso
não for atingível, registre os números em `engine/DIAGNOSTICO-P7.md` e não enfraqueça nada.

## Regras de engenharia (valem para todas as tarefas P*)

- Workspace Rust em `engine/` (edition 2021): crate de biblioteca `engine/crates/pw-engine` e binário
  `engine/crates/pw-harness`. Identificadores, comentários e mensagens de commit em inglês; docs em PT-BR.
- **Determinismo (CLAUDE.md §2, ADR-0006)**: nada de `f32`/`f64` dentro da simulação (só no harness para
  relatórios), nada de `HashMap`/`HashSet` com iteração que afete resultado (use `BTreeMap`, `Vec`
  ordenado, ids estáveis), nada de relógio, I/O ou aleatoriedade fora do PRNG versionado com sub-seeds
  por etapa (`Rng::derive(seed, "stage")`). Sem `unsafe`.
- Dependências permitidas: `serde`, `serde_json`. Nada além disso sem justificar no README do crate.
- **Nenhuma chamada de rede ou API real**: provedores de IA são traits; testes usam fixtures gravadas (CLAUDE.md §8).
- Nomes canônicos de `docs/sdd/REVISAO-CRUZADA.md` (AcceptedCommand, accepted_sequence, GroundingRef,
  DomainEvent, StateHash, WorldSnapshot, RulesetRef) e variáveis de `docs/gdd/12-variaveis-e-formulas.md`.
- Valores do GDD marcados "iniciais" ficam em constantes ou nos catálogos, nunca espalhados pelo código.
- **Testes obrigatórios** em cada tarefa (`cargo test --workspace` precisa passar). Não desative nem
  enfraqueça testes para passar.
- **Você não consegue compilar aqui** (o Windows bloqueia executáveis recém-compilados e o sandbox não
  tem rede). Escreva código Rust cuidadoso e idiomático; um script compila e testa na VPS depois e, se
  falhar, você recebe os erros numa tarefa de correção.
- Grave só com apply_patch. Não rode git. Não instale nada. Nada de segredos, IPs ou hostnames.
- Ao final rode `python tools/agents/check-encoding.py` e responda com arquivos alterados e um resumo.

- O motor existente está em `engine/` (Fase 2 concluída, 43 testes). Não quebre testes existentes nem o teste de saúde.

# Brief P10b — Governador escolhe um conjunto de comandos por turno

## Problema medido
Depois do P10, o Governador escolhe **um único comando por civilização e turno** (maior `U`). O teste de
saúde caiu de ≥ 16 para **8 cidades** em 300 turnos: quando o melhor comando é foco de cidade, ninguém
funda cidade, produz colono ou pesquisa. O GDD 09 não exige um comando por turno; ele ordena ações
candidatas dentro do Mandato.

## Tarefa
- `Governor::decide` passa a devolver um **conjunto de comandos sem conflito** por turno: no máximo um
  por "slot" (foco de cada cidade, fila de cada cidade, pesquisa, investimento, ordem de cada unidade,
  e uma ação diplomática quando existir), escolhendo em cada slot o de maior `U`, com desempate por id,
  respeitando Mandato, linhas vermelhas, reservas e a regra de ausência.
- Conflitos entre slots seguem `docs/sdd/20-matriz-de-conflitos.md` (ex.: dois comandos gastando a
  mesma reserva: só o de maior `U` entra).
- O relatório "fiz / não fiz / precisa de você" continua com 2 fatos por item.
- O teste de saúde (≥ 16 cidades, ≥ 40 população, ≤ 2 colapsos, pressão ≤ 80) volta a passar **sem
  alterar seus limites**; o teste `active_research_requests_the_selected_investment` já foi corrigido
  pelo supervisor (fundar capital antes) e deve continuar passando.

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

# Brief P2 — Estado, comandos, `step`, log, snapshot e replay

## Tarefa
Em `pw-engine` (leia `docs/sdd/01-nucleo-simulacao.md`, `03-turnos-e-sessao.md`, `09-persistencia.md`,
`20-matriz-de-conflitos.md`):
- `WorldState` (turno, seed, versão de regras, tiles, civilizações, cidades, unidades, Ledger mínimo) com
  `state_hash()` usando o `StateHasher`.
- `AcceptedCommand { command_id, world_id, turn, accepted_sequence, actor_id, origin: CommandOrigin, kind, payload }`
  com `CommandOrigin = player | governor | bot | entropy | fallback | system` e um enum `CommandKind`
  inicial: `EndTurn`, `MoveUnit`, `FoundCity`, `SetCityFocus`, `SetResearch`, `ManterPlano` (nome em
  inglês: `KeepPlan`).
- `step(state, accepted_commands, seed, versions) -> StepResult { state, events: Vec<DomainEvent>, state_hash }`:
  valida e aplica comandos na ordem de `accepted_sequence`, rejeita sem custo os inválidos (evento de
  rejeição com motivo), depois roda as fases de resolução na ordem canônica (stubs por enquanto que
  as próximas tarefas preenchem).
- `CommandLog` em memória, `WorldSnapshot` (serializável via serde_json) e `replay(snapshot, log)` que
  reproduz e compara `StateHash` por turno.
Testes: replay reproduz o hash de cada turno; comando inválido não muda o estado (só emite rejeição);
ordem de `accepted_sequence` diferente gera resultado diferente quando há conflito e igual quando não há.

## Regras de engenharia (valem para todas as tarefas P*)

- Workspace Rust em `engine/` (edition 2021): crate de biblioteca `engine/crates/pw-engine` e binário
  `engine/crates/pw-harness`. Identificadores, comentários e mensagens de commit em inglês; docs em PT-BR.
- **Determinismo (CLAUDE.md §2, ADR-0006)**: nada de `f32`/`f64` dentro da simulação (só no harness para
  relatórios), nada de `HashMap`/`HashSet` com iteração que afete resultado (use `BTreeMap`, `Vec`
  ordenado, ids estáveis), nada de relógio, I/O ou aleatoriedade fora do PRNG versionado com sub-seeds
  por etapa (`Rng::derive(seed, "stage")`). Sem `unsafe`.
- Dependências permitidas: `serde`, `serde_json` (carregar `data/catalogs/*.json`). Nada além disso sem
  justificar no README do crate.
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

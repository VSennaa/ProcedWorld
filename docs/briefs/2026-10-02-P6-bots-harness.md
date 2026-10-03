# Brief P6 — Bots T0, harness de simulação e replay

## Tarefa
- `pw-engine::bots`: bot T0 (utility AI determinística, sem LLM) por civilização: fundar cidades,
  escolher foco, pesquisar, produzir/mover unidades, explorar; ordem de envio derivada da seed do mundo
  com rotação por turno (GDD 01, ADR-0008); toda decisão gera `AcceptedCommand` com `origin = bot` e
  `GroundingRef`s dos fatos usados.
- `pw-harness` (CLI): `pw-harness run --seed N --civs 8 --turns 1000` gera o mundo, roda os bots,
  imprime `turn T hash H` a cada 100 turnos e `FINAL H`, métricas (cidades, população, pressão média,
  crises, colapsos, ms/turno) e grava `out/<seed>/log.json` e snapshots; `pw-harness replay out/<seed>`
  reproduz e confirma o mesmo hash final.
- Critério de saída da Fase 2 (CLAUDE.md §6): **1.000 turnos com 8 bots sem panic**, replay idêntico.
Testes: um teste de integração roda 200 turnos com 8 bots e verifica replay idêntico; outro garante que
duas execuções com a mesma seed dão o mesmo `FINAL`.

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

# Brief P8 — CI: testes e determinismo entre plataformas

## Tarefa
Criar `.github/workflows/engine.yml` que, em push e pull request para `develop`, `main` e `feat/**`
(filtrando por `engine/**`, `data/**` e o próprio workflow):
- job `test` (ubuntu-latest): `cargo test --workspace` em `engine/`, com cache do registry/target;
- job `determinism` em matriz `ubuntu-latest`, `windows-latest`, `macos-latest`: compila `pw-harness` em
  release, roda `run --seed 20261001 --civs 8 --turns 1000`, extrai a linha `FINAL` e publica como artefato;
- job `compare` (needs: determinism): baixa os três artefatos e falha se os hashes `FINAL` diferirem,
  imprimindo os três.
Também `engine/README.md` (PT-BR): estrutura dos crates, como rodar testes e harness (inclusive pela VPS
com `tools/dev/vps-test.sh`, explicando o bloqueio do Smart App Control no Windows), e o critério de
saída da Fase 2. Não use segredos. Não altere código Rust além do necessário para o harness rodar na CI.

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

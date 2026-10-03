# Brief P4 — Cidades, população, economia e pressão de crise

## Tarefa
Em `pw-engine` (leia `docs/gdd/03`, `04`, `06`, `12-variaveis-e-formulas.md` e `docs/sdd/15-regras-de-dominio.md`):
- fundar cidade, raio de trabalho 2, alocação automática de postos pela política (abastecer, construir,
  diversificar), rendimentos por tile (comida, produção, riqueza, conhecimento, cultura; teto 6),
  consumo de comida, celeiro, crescimento (`max(2, população)`), fome, manutenção, tesouro com teto;
- grupos híbridos (cultivadores, ofícios, mercadores) com satisfação 0–100, estabilidade local `S`;
- variáveis do GDD 12: `D` (por cidade e civilização), `G`, `W` e `E` provisórios, `P_c` e `P_civ`
  (com `(100 − S)/10`), atualização de coesão `C`, colapso parcial quando `C = 0` por 2 turnos
  (por enquanto: evento `CollapseTriggered` + congelamento simples; sucessão completa fica para depois);
- migração interna automática (1 ponto por turno, ordem estável).
Testes: estoques nunca negativos; sem comida a população cai; `P` respeita 0–100 e os limiares;
coesão nunca sai de 0–100; mesma entrada ⇒ mesmo hash.

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

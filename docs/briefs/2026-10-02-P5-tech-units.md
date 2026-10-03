# Brief P5 — Tecnologia básica, unidades, movimento e combate

## Tarefa
Em `pw-engine` (leia `docs/gdd/01`, `05`, `docs/sdd/15-regras-de-dominio.md`, `20-matriz-de-conflitos.md`,
`data/catalogs/tech_tree.json`, `units.json`):
- pesquisa: um projeto por civilização, investimento 0/10/20% da produção, conversão
  `piso(produção/2)` com teto 10, pré-requisitos, até **3 práticas ativas** com manutenção;
- unidades do catálogo: produzir na fila da cidade, mover pelo custo do bioma (rios custam extra),
  explorar; visibilidade básica (desconhecido/lembrado/visível, raio 2);
- combate em **fase própria** (GDD 01): declarar ataque reserva a unidade; perdas simultâneas pela fórmula
  do GDD 01 (`perda = min(PV, limitar(5, 40, arred(30 × F_hostil / (F_própria + F_hostil))))`, inteiros);
  controle muda só se restar coalizão hostil.
Testes: tecnologia sem pré-requisito é rejeitada; 4ª prática ativa é rejeitada; dois ataques no mesmo
hex formam um confronto; resultado de combate independe da ordem de declaração; determinismo.

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

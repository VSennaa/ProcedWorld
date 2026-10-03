# Brief P7b — Terminar a correção da simulação (expansão e pressão)

## Situação
O P7 já corrigiu várias causas (ver `engine/DIAGNOSTICO-P7.md`). Medição atual na VPS (seed 20261001,
8 civilizações, 300 turnos):
`cities 13 population 52 average_pressure 7 crises 0 collapses 1` — por civilização: 5 civs com 2 cidades,
3 com 1 cidade; coesão 26–30; a civ 7 colapsou com coesão 0.
O teste de saúde `health_simulation_expands_grows_and_avoids_systemic_collapse` exige cidades ≥ 16,
população ≥ 40, colapsos ≤ 2 e pressão média entre 10 e 80. Faltam **cidades (13 → ≥ 16)** e
**pressão (7 → ≥ 10)**.

## Tarefa
1. Expansão: descobrir por que 3 civilizações não fundam a 2ª cidade e por que ninguém passa de 2 em 300
   turnos (custo/tempo do colono, escolha de local, pesquisa, bot que não repete a expansão, falta de tile
   válido). Corrigir causa, não só ajustar números.
2. Pressão muito baixa e coesão baixa ao mesmo tempo indicam que `P` não está recebendo os termos do
   GDD 12 (privação, tensão de grupos, estabilidade local com peso reduzido). Verificar a fórmula contra
   `docs/gdd/12-variaveis-e-formulas.md` e corrigir.
3. Atualizar `engine/DIAGNOSTICO-P7.md` com as novas causas.
Não enfraqueça os limites do teste de saúde; se algum for impossível pelas regras do GDD, explique no
diagnóstico e pare no limite mais próximo justificável.

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

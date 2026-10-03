# Brief P7 — Diagnosticar e corrigir a simulação degenerada

## Problema medido (VPS, release)
`pw-harness run --seed 20261001 --civs 8 --turns 1000` termina com:
`cities 8 population 8 average_pressure 60 crises 0 collapses 8` — nenhuma civilização expandiu nem
cresceu (1 cidade com 1 de população cada) e **todas colapsaram**. O determinismo e o replay estão
corretos; o problema é de regras/bots/balanceamento.

## Tarefa
1. Diagnosticar a causa raiz lendo o código de `engine/crates/pw-engine` (bots, cidades, economia,
   pressão/coesão, colapso, fundação de cidades e produção de colonos). Escreva o diagnóstico em
   `engine/DIAGNOSTICO-P7.md` (causas encontradas, com arquivo e função).
2. Corrigir as causas sem violar o GDD (`docs/gdd/03`, `04`, `06`, `12`) — por exemplo: crescimento que
   nunca dispara, colonos nunca produzidos ou nunca usados, manutenção maior que a renda desde o turno 1,
   coesão caindo sem causa, colapso que dispara cedo demais. Valores iniciais podem ser ajustados nos
   catálogos/constantes, citando o motivo no diagnóstico.
3. Acrescentar um **teste de saúde** de integração (seed fixa, 8 civilizações, 300 turnos) que exige:
   total de cidades ≥ 16, população total ≥ 40, no máximo 2 colapsos, e pressão média entre 10 e 80.
   **Não enfraqueça esses limites** para passar; se algum for impossível pelas regras do GDD, explique no
   diagnóstico e pare no limite mais próximo justificável.
4. Fazer o harness imprimir também, por civilização no fim: cidades, população, coesão, colapsado.

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

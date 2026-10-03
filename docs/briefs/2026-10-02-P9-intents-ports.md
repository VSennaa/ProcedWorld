# Brief P9 — Intenções e portas de IA (T0/T1/T2)

Leia `docs/sdd/04-camadas-de-ia.md`, `docs/sdd/08-memoria-e-contexto.md`, CLAUDE.md §2–3.
- Módulo `pw-engine::ai`: `ActionIntent` (envelope com `request_id`, ator, turno, versão, `kind`,
  parâmetros, `grounding: Vec<GroundingRef>`), validação (schema, faixas, Mandato quando houver,
  grounding: todo `GroundingRef` precisa existir no estado e ser visível ao ator) e conversão em
  `AcceptedCommand` com `origin` correto.
- Traits `DecisionPort` (T1: choice/score/yes_no com probabilidade, como Jev/Laya), `LlmPort` (T2:
  texto → intenção tipada) e `MemoryPort`; implementação `FixturePort` que lê respostas gravadas (JSON) e
  `NullPort` que sempre falha (força T0). Timeout/erro/saída inválida ⇒ fallback T0 determinístico.
- `IntentEvidence` gravada junto do comando (para auditoria), nunca usada no replay.
- Texto de jogador é dado não confiável: tipo `UntrustedPlayerText` que nunca vira instrução nem fato.
Testes: intenção sem grounding é rejeitada; grounding de fato não visível é rejeitado; `NullPort` ⇒ T0
produz comando válido; fixture gravada ⇒ mesmo comando; replay não consulta portas.

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

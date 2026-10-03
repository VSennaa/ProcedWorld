# Brief P16 — Ordens de unidade, bloqueio de Pronto e catálogo no servidor

Branch: `feat/engine/unit-orders` (a partir de `feat/ai/phase3-core`). Área: `engine/` apenas.

## Contrato (já aprovado, não reinterpretar)

- SDD 15 §4.3 (ordens de unidade), SDD 10 §5.1 e §5.2 (erro `units_awaiting_orders`, mensagem
  `catalog`, `protocol_version: "1.0"`), SDD 03 "Decidido" (bloqueio só para humanos presentes).
- GDD 01 "Decidido": unidades ociosas bloqueiam Pronto; ordens de vários turnos não.

## Entregas

1. `pw-engine`:
   - `UnitOrder { Idle, Fortify, Explore, MoveTo { target } }` e `skipped_turn: Option<u32>`
     em `UnitState`, ambos no hash de estado;
   - comandos `SetUnitOrder { unit_id, order }` e `SkipUnit { unit_id }`, validados (dono, unidade
     viva, alvo legal) com `RejectionReason` existente ou nova;
   - execução das ordens persistentes no `step`: `MoveTo` avança e volta a `Idle` ao chegar ou se o
     destino ficar inalcançável; `Explore` reaproveita a lógica atual de exploração e volta a `Idle`
     sem alvo; `Fortify` persiste;
   - `pub fn idle_units(state, civ) -> Vec<UnitId>` (ordem de ID);
   - bots T0 e Governador passam a usar `SetUnitOrder` onde fizer sentido (batedor → `Explore`,
     defesa → `Fortify`), mantendo o determinismo; ajuste os testes que dependem disso.
2. `pw-server`:
   - `ready` de humano presente com `idle_units` não vazio ⇒ erro `units_awaiting_orders` com os
     IDs no `detail`;
   - visão (`view.rs`) com `idle_units` e, por unidade própria, `order` e `skipped_turn`;
   - frame `catalog` enviado logo após `join` (tecnologias: id, nome, custo, pré-requisitos;
     unidades: id, nome, papel, movimento, força; hash/versão do catálogo);
   - `protocol_version` como texto `"1.0"` na entrada e na saída; aceitar major `1`.
3. Testes: determinismo (mesmo hash em replay com as novas ordens), `MoveTo` chega e vira `Idle`,
   `SkipUnit` libera só no turno atual, `ready` recusado e depois aceito, `catalog` após `join`.
   O harness de 1000 turnos com 8 civilizações continua passando.
4. Atualizar `engine/crates/pw-server/README` ou comentário de módulo se o contrato mudar.

## Regras

- Rust só compila na VPS: `VPS_SSH=<passado no prompt> tools/dev/vps-test.sh`. Espaçar chamadas SSH
  em ≥ 60 s. Nunca escrever IP/host em arquivos.
- Inteiros, `BTreeMap`, sem relógio nem I/O no `step`.
- Commits Conventional (`feat(engine): ...`), identidade do CLAUDE.md, `git add <arquivos>`.
- Quadro: `python tools/board/update.py P16 doing|done|failed --note "..."`.

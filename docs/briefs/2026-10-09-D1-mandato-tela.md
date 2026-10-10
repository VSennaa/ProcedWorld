# Brief D1 — Mandato: escolha de preset pelo jogador (servidor + cliente)

Áreas: `engine/crates/pw-server/src/` (`world.rs`, `view.rs`, protocolo/handler da sessão) e `client/`.
Contrato: GDD 09 (Governador e Mandato), SDD 12 (barra de telas), SDD 10 (Governador). Estado atual:
`world.rs` fixa `Mandate::preset(MandatePreset::GrowCautiously, 1)` para toda civilização; não há
mensagem para mudá-lo nem tela. Decidido em 2026-10-09, por delegação do usuário (memória
`autonomia-de-design`): o jogador escolhe entre os 3 presets existentes (`Balanced`, `Recover`,
`GrowCautiously`); edição campo a campo fica para depois.

## D1a — Servidor
- Nova mensagem do cliente `set_mandate { preset }` (só da própria civilização, autenticada como as
  demais). Valida o preset, incrementa `version` do Mandato, grava no log de comandos/persistência como
  as outras entradas externas (replay deve reproduzir) e responde com a visão atualizada.
- A visão da civilização ganha `mandate { preset, version, direction, limits }` com o que o jogador
  precisa ler. Nunca expor o Mandato de outras civilizações (teste de não vazamento).
- Mundos restaurados do disco mantêm o Mandato escolhido (teste de recarga).

## D1b — Cliente
- Tela **Mandato** na barra (ao lado de Relações/Crônica): mostra o preset atual e a descrição em PT-BR
  do que o Governador fará e não fará quando o jogador estiver ausente; três botões para trocar.
- Troca só envia `set_mandate`; o estado mostrado vem da visão do servidor (nada de estado otimista).
- Sem botão sem efeito. Alvos ≥ 44 px. UI PT-BR, código em inglês.

## Aceite
- `cargo test` do workspace verde na VPS (`flock`), testes headless do cliente verdes, e um teste ponta
  a ponta contra servidor real (estender `client/tests/real_server.gd` ou novo arquivo): trocar o preset
  e ver `version` subir.
- Atualizar SDD 10/12 se o contrato mudar. Não reformatar arquivos, não tocar em `project.godot`.
- Não rodar comandos git.

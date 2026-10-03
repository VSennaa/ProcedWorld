# Brief A3 — Cliente do alfa mini

Área: `client/` apenas. Contrato: SDD 12 "Alfa mini (2026-10-03)", SDD 10 §5.2 (catálogo com
`event_templates` e `improvements`), SDD 15 §4.2.2 (ordem `Build`). Tarefas em sequência, cada uma
com testes headless verdes antes da próxima.

## A3a — Pesquisa e Sociedade
- Na árvore: tocar numa tecnologia disponível escolhe a pesquisa (`set_research` com o id; confira a
  forma serde em `engine/crates/pw-engine/src/world.rs`). Mostrar atual, custo e turnos estimados.
- Tela **Sociedade** (hoje placeholder) só leitura: `C`, `L`, `S`, `D`, `G`, `W`, `E`, `P` da
  civilização e por cidade, com nomes e escalas de `docs/gdd/12-variaveis-e-formulas.md`; `P` com
  fatores, nunca barra opaca; `W` e `E` marcados como provisórios.

## A3b — Eventos legíveis
- Eventos pendentes mostram nome, categoria, texto e rótulos das escolhas do `catalog`
  (`event_templates`); fallback para o id humanizado se faltar. Responder pela Pauta e por uma tela
  de evento acessível pelo botão de próxima decisão.

## A3c — Atacar e construir melhoria
- **Atacar**: com unidade própria selecionada e unidade estrangeira visível adjacente, ação Atacar
  (`declare_attack` com `attacker`/`target`).
- **Construir melhoria**: trabalhador mostra as melhorias possíveis no tile (tecnologia dominada,
  bioma do catálogo; o servidor decide) e envia a ordem `{"type":"build","data":{"improvement":id}}`.
  Mapa desenha melhoria do tile (`improvement`) e obra (`build_progress`) quando a visão trouxer.

## Regras
- UI PT-BR, código em inglês, alvos ≥ 44 px, nada de botão sem efeito; adaptador tolerante a campos
  ausentes. Atualize fixtures, testes (`run_tests.gd`, `live_flow.gd`) e capturas em `client/docs/`.

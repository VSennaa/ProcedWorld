# Brief P17 — Cliente ligado ao servidor, unidades no mapa e árvore de pesquisa

Branch: `feat/front/live-client` (a partir de `feat/ai/phase3-core`). Área: `client/` apenas.

## Contrato (já aprovado, não reinterpretar)

- SDD 12 "Unidades, Pronto e árvore de pesquisa", SDD 10 §3, §5.1 e §5.2, SDD 15 §4.3.
- Formato real da visão: `engine/crates/pw-server/src/view.rs` (`view_for`) e mensagens em
  `engine/crates/pw-server/src/ws.rs`. Em paralelo, o P16 acrescenta à visão `idle_units`, e por
  unidade própria `order` (`"Idle" | "Fortify" | "Explore" | {"MoveTo":{"target":N}}`) e
  `skipped_turn`; acrescenta o frame `catalog` após `join` e o erro `units_awaiting_orders`.
  Programe contra esse contrato; comandos novos: `SetUnitOrder { unit_id, order }`, `SkipUnit { unit_id }`.

## Entregas

1. Tela inicial: URL do servidor (padrão `ws://127.0.0.1:8100/ws`), **Criar mundo** (seed,
   civilizações) e **Entrar** (mundo, civilização, token guardado para reconexão), mais
   **Demonstração** que abre a fixture atual.
2. Adaptador da visão do servidor para o modelo do cliente (tiles, cidades, unidades, civilização,
   eventos pendentes). Nova fixture `fixtures/server_view.json` no formato real do `view_for` (com
   os campos do P16) usada nos testes.
3. Unidades no mapa: ícone por papel (`assets/entities`), cor da civilização; seleção mostra tipo,
   movimento, vida e ordem com ações **Mover** (toque no destino ⇒ `MoveTo`), **Explorar**,
   **Fortificar**, **Pular**.
4. Pauta: fila de unidades ociosas (toque centraliza no mapa); **Pronto** desabilitado com
   "N unidades aguardam ordem"; tratar o erro `units_awaiting_orders`.
5. Árvore de pesquisa só leitura a partir do `catalog`: colunas por profundidade de pré-requisito,
   dominadas, atual e progresso.
6. Testes headless (`run_tests.gd`) para o adaptador, fila de ociosas, gate do Pronto e layout da
   árvore. Retrato, mobile primeiro, alvos de toque ≥ 44 px.

## Regras

- Godot 4.7.2 no PC. Testes:
  `"/c/Users/vinic/Downloads/Godot_v4.7.2-stable_win64.exe/Godot_v4.7.2-stable_win64_console.exe" --headless --path client -s res://tests/run_tests.gd`
- Não versionar `*.import` gerados nem mudanças acidentais de `project.godot`, salvo se necessárias.
- Texto de UI em PT-BR; código e comentários em inglês. Nada de botões que não fazem nada.
- Commits Conventional (`feat(front): ...`), identidade do CLAUDE.md, `git add <arquivos>`.
- Quadro: `python tools/board/update.py P17 doing|done|failed --note "..."`.

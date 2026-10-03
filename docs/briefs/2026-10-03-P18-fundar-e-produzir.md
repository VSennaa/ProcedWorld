# Brief P18 — Fundar cidades e produzir unidades no cliente ao vivo

Branch: `feat/front/found-and-produce` (a partir de `feat/ai/phase3-core`). Áreas: `client/` e
`engine/crates/pw-server/` (só o necessário na visão/protocolo).

## Problema (achado no teste ponta a ponta de 2026-10-03)

Um humano entra num mundo novo sem cidade e sem unidade. O motor permite fundar a primeira cidade
sem colono (`FoundCity` em `world.rs`, `founding_with_settler`), mas o cliente não tem essa ação,
nem fundar com colono, nem fila de produção. O jogador só consegue passar turnos.

## Contrato

- GDD 01 e 04 (cidades produzem unidades em fila, como no *Civilization*); SDD 12 (Mapa: ordem
  espacial sobre alvo legal; Pauta: o que exige atenção); SDD 10 (comandos via `submit_command`).
- Comandos do motor já existentes: `FoundCity { city_id, target }`, `QueueUnit { city_id, unit_type }`.
  Veja como o servidor/motor esperam `city_id` (quem gera o id) e siga isso; se o cliente precisar
  propor o id, documente a regra no SDD 10 §6 e garanta que conflito vira rejeição limpa.

## Entregas

1. Servidor: a visão inclui `home_tile` (ponto inicial) enquanto a civilização não tem cidade, e o
   que mais for estritamente necessário (ex.: próximo `city_id` livre, se o cliente precisar). Teste.
2. Cliente:
   - Pauta: cartão de atenção "Fundar a capital" quando não há cidade; toque centraliza no ponto
     inicial; ação **Fundar capital** ali.
   - Unidade colono: ação **Fundar cidade** no tile atual (só quando legal pelo que o cliente sabe;
     o servidor decide).
   - Cidade própria: tocar abre painel com população, foco, fila de produção e progresso, e a lista
     de unidades que podem ser produzidas (tecnologia exigida dominada, catálogo) ⇒ `QueueUnit`.
   - Testes headless para as novas ações e o painel; atualizar `tests/live_flow.gd` se cobrir.
3. Ponta a ponta: estender `client/tests/real_server.gd` para fundar a capital no turno 0, enfileirar
   uma unidade, avançar turnos (dando ordem às ociosas) até a unidade aparecer e dar ordem a ela.
   Rodar contra o servidor real (instruções no prompt) e registrar o resultado.
4. Capturas atualizadas em `client/docs/` (Pauta com o cartão de capital, painel da cidade).

## Regras

- Rust só compila/roda na VPS (`tools/dev/vps-test.sh`); SSH espaçado ≥ 60 s; nunca IP/host em arquivo.
- UI PT-BR, código em inglês, alvos de toque ≥ 44 px, nada de botão sem efeito.
- Commits Conventional, identidade do CLAUDE.md, `git add <arquivos>`. Quadro: `P18`.

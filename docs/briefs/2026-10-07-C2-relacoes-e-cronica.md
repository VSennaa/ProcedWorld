# Brief C2 — Telas Relações e Crônica (só leitura)

Áreas: `client/` e, na parte do servidor, só `engine/crates/pw-server/src/view.rs` (+ teste).
Contrato: SDD 12 (barra Pauta/Mapa/Sociedade/Relações/Crônica), SDD 07 (estados diplomáticos e
Ledger), SDD 16 (visibilidade), GDD 07.

## C2a — Servidor: relações na visão
- A visão da civilização ganha `relations`: uma entrada por civilização **conhecida** (que teve contato,
  conforme o motor), com `civ`, `state` (máquina de estados de `diplomacy.rs`), `confidence` (Cf),
  `resentment` (R), `debt` (Dv) e as últimas N entradas do Ledger **que envolvem a própria civilização**
  (tipo, turno, contraparte). Nunca as relações entre terceiros. Teste de não vazamento.

## C2b — Cliente: Relações
- Lista as civilizações conhecidas com estado, Cf/R/Dv com escala (GDD 12) e as entradas recentes do
  Ledger legíveis em PT-BR. Só leitura (propostas ficam para depois): nada de botão sem efeito.

## C2c — Cliente: Crônica
- Histórico local dos turnos recebidos na sessão: por turno, eventos do relatório (`events_for`:
  comandos aplicados/rejeitados, migrações, colapsos, diplomacia resolvida) e eventos da Entropia
  respondidos, em PT-BR. Marcar como "nesta sessão" (o servidor ainda não guarda a crônica).

Regras: UI PT-BR, código em inglês, alvos ≥ 44 px, testes headless verdes, não reformatar arquivos.

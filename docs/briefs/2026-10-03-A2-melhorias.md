# Brief A2 — Melhorias de terreno no motor (trabalhador)

Área: `engine/` (motor e visão do servidor). Tarefa delicada: regras determinísticas.
Contrato: SDD 15 §4.2.2 (e §4.3 ordens), GDD 02 (melhoria 0/1 por tile), GDD 03 (limite 0–6 por
rendimento, manutenção e prioridade de pagamento). Catálogo: `data/catalogs/improvements*.json`.

## Entregas

1. Estado do tile: `improvement: Option<String>` e obra em andamento (`build_progress`), no hash.
2. `UnitOrder::Build { improvement }` (wire `{"type":"build","data":{"improvement":"improvement.farm"}}`):
   só unidade `worker`; valida tecnologia, bioma (mapeie os biomas do catálogo para os terrenos do
   motor de forma explícita e testada), tile sem melhoria nem outra obra, tile próprio ou neutro
   adjacente ao território. Recurso exigido pelo catálogo: se o motor não modela recursos de tile,
   documente a simplificação no SDD 15 e no `engine/DIAGNOSTICO-P7.md` (seção "Melhorias").
3. Progresso por turno com a ordem ativa; ao concluir, aplica `improvement`, unidade volta a `Idle`.
   Ritmo proposto: trabalho fixo por turno; custos não-produção (ex.: pedra) — se não existirem no
   motor, registre como simplificação provisória.
4. Efeitos de rendimento do catálogo somados ao rendimento do tile, respeitando 0–6; manutenção em
   riqueza na ordem de pagamento existente.
5. Visão do servidor: tile com `improvement` e `build_progress`. Bots T0: trabalhador ocioso constrói
   a melhoria legal mais útil perto de cidade (determinístico).
6. Testes: validação, progresso, conclusão, efeito no rendimento, limite 6, manutenção, determinismo e
   replay; harness 300 turnos sem regressão grave (registre a linha de métricas).

# Retrabalho por executor

> Gerado por `python tools/agents/rework.py report` a partir de `retrabalho.jsonl`. Não editar à mão.

Retrabalho = reprovações do juiz + correções automáticas (loop de build) + execuções extras do agente para corrigir achados + correções feitas à mão pelo supervisor + trocas de executor.

| Executor | Tarefas | Mescladas | Retrabalho médio | 1ª nota média do juiz | Nota final média | Correções do supervisor | Trocas de executor | Custo direto |
|---|---|---|---|---|---|---|---|---|
| claude-opus | 2 | 2 | 2.0 | 6.0 | 8.5 | 1 | 0 | cota da assinatura |
| claude-sonnet | 7 | 7 | 0.3 | 8.5 | 8.5 | 2 | 0 | cota da assinatura |
| codex | 16 | 11 | 2.4 | 5.0 | 6.8 | 6 | 4 | cota da assinatura |
| opencode | 1 | 1 | 1.0 | 7.0 | 7.0 | 0 | 0 | US$ 0.47 |

## Tarefas

| Data | Tarefa | Executor / modelo | Crítica | Juiz (notas) | Retrabalho | Resultado | Observações |
|---|---|---|---|---|---|---|---|
| 2026-10-02 | P1 | codex / gpt-5.6-terra/gpt-6-luna (Jev) | sim | — | 0 | merged |  |
| 2026-10-02 | P2 | codex / gpt-5.6-terra/gpt-6-luna (Jev) | sim | — | 0 | merged |  |
| 2026-10-02 | P3 | codex / gpt-5.6-terra/gpt-6-luna (Jev) | sim | — | 0 | merged |  |
| 2026-10-02 | P4 | codex / gpt-5.6-terra/gpt-6-luna (Jev) | sim | — | 0 | merged |  |
| 2026-10-02 | P5 | codex / gpt-5.6-terra/gpt-6-luna (Jev) | sim | — | 1 | merged |  |
| 2026-10-02 | P6 | codex / gpt-5.6-terra/gpt-6-luna (Jev) | sim | — | 0 | merged |  |
| 2026-10-02 | P7 | codex / gpt-5.6-terra/gpt-6-luna (Jev) | sim | — | 4 | failed | 3 correções sem passar; refeito como P7b |
| 2026-10-02 | P7b | codex / gpt-5.6-terra/gpt-6-luna (Jev) | sim | — | 3 | failed | 3 correções sem passar; diagnóstico manual (DIAGNOSTICO-P7) |
| 2026-10-02 | P8 | codex / gpt-5.6-terra/gpt-6-luna (Jev) | sim | — | 0 | merged | 2 execuções anteriores com exit 1 (falha de execução) |
| 2026-10-02 | P9 | codex / gpt-5.6-terra/gpt-6-luna (Jev) | sim | — | 1 | merged |  |
| 2026-10-02 | P10 | codex / gpt-5.6-terra/gpt-6-luna (Jev) | sim | — | 4 | failed | Governador com 1 comando/turno; refeito por Sonnet (P10b) |
| 2026-10-02 | P13 | codex / gpt-5.6-terra/gpt-6-luna (Jev) | sim | — | 0 | merged |  |
| 2026-10-02 | P12b | codex / gpt-5.6-terra/gpt-6-luna (Jev) | sim | — | 3 | parked | piso de pressão inatingível; virou decisão de design |
| 2026-10-02 | P10b/P11 (quota) | codex / gpt-5.6-terra | não | — | 3 | failed | cota do Codex esgotada no meio; loop girou sem trabalho útil |
| 2026-10-03 | P16 | claude-sonnet / sonnet-5.5 | sim | — | 1 | merged | supervisor corrigiu ociosidade (sobra de movimento) |
| 2026-10-03 | P17 | claude-sonnet / sonnet-5.5 | não | — | 1 | merged | supervisor alinhou formato de ordens com o servidor |
| 2026-10-03 | P18 | claude-sonnet / sonnet-5.5 | não | — | 0 | merged |  |
| 2026-10-03 | P19 | claude-sonnet / sonnet-5.5 | não | — | 0 | merged |  |
| 2026-10-03 | P20 | claude-sonnet / sonnet-5.5 | sim | — | 0 | merged |  |
| 2026-10-03 | P21 | claude-sonnet / sonnet-5.5 | sim | — | 0 | merged |  |
| 2026-10-03 | A1 persistência+catálogo | codex / gpt-5.6-terra (Jev: gpt-6.1-sol) | sim | 5✗ → 6✗ | 9 | merged | reformatou world.rs (2380 linhas) e editou tools/; supervisor reaplicou o mínimo, corrigiu chave do catálogo e a DSL; 3 correções automáticas perdidas por erro do supervisor (VPS_SSH); último bloqueante do juiz estava errado |
| 2026-10-03 | A2 melhorias | claude-opus / opus-5.5 | sim | 6✗ → 5✗ → 8.5 | 4 | merged | bug sutil de manutenção (agente corrigiu); vazamento de névoa (supervisor corrigiu) |
| 2026-10-03 | A3 cliente alfa mini | codex / gpt-5.6-terra (Jev: gpt-6.1-sol) | não | 5✗ → 6✗ → 7.5 | 10 | merged | capturas travaram no headless (supervisor fez); cota do Codex acabou na A3e (Sonnet terminou); polimento final por Sonnet |
| 2026-10-07 | B1 pressão (medição + W) | claude-opus / opus-5.5 | sim | — | 0 | merged | parou corretamente: meta exigia decisão de design |
| 2026-10-07 | C1 segurança da persistência | claude-sonnet / sonnet-5.5 | sim | 8.5 | 0 | merged |  |
| 2026-10-07 | C2 Relações e Crônica | opencode / deepseek-flash (Jev: gpt-6.1-sol) | não | 7 | 1 | merged | 3 subtarefas + juiz + 1 correção por US$ 0,47; sem reformatação; ressalvas reais corrigidas |

✗ = reprovado. Custo direto: só o que é cobrado por uso (DeepSeek pré-pago); Codex e Claude gastam cota da assinatura (janela de 5 h e semanal).

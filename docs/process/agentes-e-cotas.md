# Agentes, subagentes e auditoria de cotas

> Como o desenvolvimento é distribuído entre agentes e como o projeto se protege contra ficar
> **congelado** por falta de cota. Vale para sessões no PC e na VPS.

## 1. Papéis

| Agente | Papel | Cota que consome |
|---|---|---|
| **Claude** (Claude Code) | Supervisor: planeja, escreve prompts, revisa, faz commit e push, conversa com o usuário | Plano Claude (janela de 5 h e semanal) |
| **Subagentes Codex roteados pelo Jev** | Executores de tarefas independentes (rascunhos, implementação de arquivos isolados) | Créditos do **OpenRouter** (`OPENROUTER_API_KEY`) |
| **Codex com conta ChatGPT** | Executor alternativo quando o OpenRouter estiver sem crédito | Plano ChatGPT/Codex (janela de 5 h e semanal) |
| **opencode + DeepSeek** | Último recurso, mais barato | Créditos DeepSeek |

Subagentes **nunca** fazem commit, push ou trocam de branch (ver `AGENTS.md`). Só o supervisor integra.

## 2. Subagentes com o Jev escolhendo o modelo

Ferramenta: `tools/agents/jev-codex.sh <tarefa> <arquivo-de-prompt> [read-only|workspace-write]`.
Skill do Claude: `.claude/skills/jev-subagents/`.

1. O script envia a tarefa ao **Jev Router** (`typesafe/jev-router`, OpenRouter) com `max_tokens: 1`.
   O Jev decide o modelo e o informa no campo `model` da resposta. O custo é só o da decisão.
2. Confere se o modelo escolhido aceita *tools*; se não, usa `JEV_FALLBACK_MODEL`
   (padrão `deepseek/deepseek-v4-flash`).
3. Roda `codex exec` com esse modelo via OpenRouter, do começo ao fim da tarefa.
4. Registra a decisão em `.agent-runs/decisions.jsonl` (fora do git): tarefa, modelo, custo da decisão,
   fallback, saída, tokens, duração. Esses dados também servem de calibração inicial da camada T1
   do jogo (ADR-0005).

**Por que um modelo por tarefa**: testado em 2026-10-01, deixar o Jev Router rotear cada requisição
de uma sessão do Codex falha já na segunda chamada ("No models satisfy the decisions policy"),
porque o histórico carrega itens de raciocínio específicos do modelo anterior.

**Custo**: o Jev escolhe modelos de preços muito diferentes. Na primeira rodada, a escolha variou entre
`openai/gpt-6-luna`, `openai/gpt-6.1-sol` e `anthropic/claude-sonnet-5.5`; a decisão em si custou
entre US$ 0,0002 e US$ 0,004. O custo da execução depende do modelo escolhido; sempre audite o saldo
antes de disparar lotes.

## 3. Auto-auditoria de cotas (o projeto não pode congelar)

### Fontes

| Cota | Como ler |
|---|---|
| Claude | ferramenta `get_usage` do app desktop (`mcp__ccd_session_mgmt__get_usage`): janelas de 5 h e semanal, `percentUsed`, `resetsAt` |
| Codex (ChatGPT) | `tools/agents/quota-check.sh` → `codex`: último snapshot de `rate_limits` gravado em `~/.codex/sessions/`. Só é atualizado quando o Codex roda pela conta ChatGPT; respeite `snapshot_age_min` (snapshot antigo = valor mínimo, não exato) |
| OpenRouter | `tools/agents/quota-check.sh` → `openrouter`: `usage_usd`, `limit_usd`, `limit_remaining_usd`, `usage_daily_usd` |

Use sempre a **maior** porcentagem entre a janela de 5 h e a semanal.

### Quando auditar

- No início de toda sessão (junto com a leitura de `docs/STATUS.md`).
- Antes de disparar qualquer lote de subagentes.
- Ao fim de cada tarefa grande e antes de encerrar a sessão.

### Regras de decisão

| Situação | Ação |
|---|---|
| Claude < 75% | Claude trabalha normalmente; delega o que for paralelizável |
| Claude ≥ 75% | Claude passa a só supervisionar: escreve o handoff em `docs/STATUS.md`, faz commit e delega a próxima tarefa |
| OpenRouter com saldo < US$ 0,50, ou abaixo do custo estimado do lote | Não disparar subagentes Jev; usar Codex com conta ChatGPT |
| Codex (ChatGPT) ≥ 90% ou limite atingido | Usar opencode + DeepSeek |
| Todos os executores esgotados | Registrar estado e próximos passos em `docs/STATUS.md`, commit e push, e **agendar a retomada** para o menor `resetsAt` + 2 min (tarefa agendada única) |
| Claude chegou a 95% durante uma tarefa | Parar em ponto seguro, commit, push, handoff e agendar a retomada |

**Regra de ouro**: nenhuma sessão termina sem (a) o estado commitado e enviado ao remoto e
(b) um próximo passo claro em `docs/STATUS.md` ou uma retomada agendada. Projeto parado sem
retomada agendada é bug de processo.

### Registro

Cada auditoria relevante (troca de executor, pausa, retomada agendada) é anotada em
`docs/STATUS.md` → seção "Agora", com os números lidos e o horário de retomada.

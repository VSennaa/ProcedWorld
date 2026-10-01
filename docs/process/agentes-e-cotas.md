# Agentes, subagentes e auditoria de cotas

> Como o desenvolvimento é distribuído entre agentes e como o projeto se protege contra ficar
> **congelado** por falta de cota. Vale para sessões no PC e na VPS.

## 1. Papéis

| Agente | Papel | Cota que consome |
|---|---|---|
| **Claude** (Claude Code) | Supervisor: planeja, escreve prompts, revisa, faz commit e push, conversa com o usuário | Plano Claude (janela de 5 h e semanal) |
| **Subagentes Codex com modelo escolhido pelo Jev** | Executores de tarefas independentes (rascunhos, implementação de arquivos isolados) | Execução: plano **ChatGPT/Codex** (janela de 5 h e semanal). Decisão do Jev: centavos de crédito do **OpenRouter** |
| **opencode + DeepSeek** | Último recurso, mais barato | Créditos DeepSeek |

Subagentes **nunca** fazem commit, push ou trocam de branch (ver `AGENTS.md`). Só o supervisor integra.

## 2. Subagentes com o Jev escolhendo o modelo

Ferramenta: `tools/agents/jev-codex.sh <tarefa> <arquivo-de-prompt> [read-only|workspace-write]`.
Skill do Claude: `.claude/skills/jev-subagents/`.

**O OpenRouter serve só para a decisão do Jev. Nenhum modelo executa tarefas pelo OpenRouter.**
O crédito de lá existe para as decisões, que custam entre US$ 0,00001 e US$ 0,002 cada.

1. O script envia o início da tarefa (2.000 caracteres) ao **Jev Router** (`typesafe/jev-router`)
   com `max_tokens: 1`. O Jev decide o modelo e o informa no campo `model` da resposta.
2. O Jev escolhe entre todos os modelos do OpenRouter e não aceita restrição. A escolha é mapeada para
   um modelo do Codex **por capacidade**, usando o benchmark de LLMs do Fabio Akita, guardado em
   [`tools/agents/model-scores.json`](../../tools/agents/model-scores.json) (fonte e data no arquivo):
   - **exato** se o Jev escolheu um modelo `openai/*` que o Codex tem;
   - senão, o **modelo do Codex mais barato com nota ≥ à nota do modelo escolhido pelo Jev**;
   - se o modelo do Jev não está no benchmark, a nota é estimada pelo modelo do benchmark com preço de
     saída mais próximo no OpenRouter (escala logarítmica).

   Com o benchmark v4 (2026-09-23): `claude-opus-5.5` (100) → `gpt-5.6-terra` (100, US$ 9,52);
   `kimi-k3` (85) e `deepseek-v4-flash` (86) → `gpt-6-luna` (95,5, US$ 0,84). O `gpt-6-sol` (91) só é
   usado se o Jev o escolher exatamente, porque o `gpt-6-luna` tem nota maior e custa menos.

   **Limitação**: o benchmark mede vigilância de segurança de um agente de código em Rails, não escrita
   de documentos. É um indicador de capacidade geral de agente, não verdade absoluta. Atualizar o JSON
   quando o Akita publicar uma nova versão.
2b. **Escolha da Anthropic (Opus, Sonnet, Fable, Haiku) roda no próprio Claude**, desde que o Claude
   tenha **pelo menos 20% de cota livre** na maior das janelas (5 h ou semanal), reservados para
   supervisionar o Codex. O supervisor checa `get_usage`; se houver folga, exporta `JEV_CLAUDE_OK=1`.
   O script então não roda o Codex: registra a decisão (`executor: claude`) e sai com código 76, e o
   Claude executa a tarefa com a ferramenta Agent no modelo correspondente. Sem folga, a escolha é
   mapeada para o Codex normalmente.
3. O Codex roda a tarefa inteira com esse modelo **pela conta do ChatGPT**.
4. Sem crédito para a decisão (abaixo de `JEV_MIN_CREDIT`, padrão US$ 0,10) ou com erro no Jev, usa
   `JEV_FALLBACK_MODEL` (padrão `gpt-6-sol`) — o roteador nunca para o projeto.
5. Registra tudo em `.agent-runs/decisions.jsonl` (fora do git): escolha do Jev, modelo do Codex,
   tipo de mapeamento, custo da decisão, saída, tokens, duração. Esses dados também servem de
   calibração inicial da camada T1 do jogo (ADR-0005).

**Por que um modelo por tarefa**: testado em 2026-10-01, deixar o Jev Router rotear cada requisição
de uma sessão do Codex falha já na segunda chamada ("No models satisfy the decisions policy"),
porque o histórico carrega itens de raciocínio específicos do modelo anterior.


**Incidente de 2026-10-01 (primeiro lote, 12 pilares do GDD, 4 em paralelo)**: 2 pilares concluídos,
o resto falhou com `402 Payment Required` com US$ 3 de saldo, porque o OpenRouter reserva o custo
máximo de cada requisição em andamento. Os prompts pediam para ler todos os pilares, e uma execução
chegou a 412 mil tokens. Correções: a execução saiu do OpenRouter e passou para o Codex na conta do ChatGPT (o OpenRouter ficou
só com a decisão); no máximo 2 em paralelo; prompts citam só os arquivos necessários.

## 3. Auto-auditoria de cotas (o projeto não pode congelar)

### Fontes

| Cota | Como ler |
|---|---|
| Claude | ferramenta `get_usage` do app desktop (`mcp__ccd_session_mgmt__get_usage`): janelas de 5 h e semanal, `percentUsed`, `resetsAt` |
| Codex (ChatGPT) | `tools/agents/quota-check.sh` → `codex`: último snapshot de `rate_limits` gravado em `~/.codex/sessions/`. Só é atualizado quando o Codex roda pela conta ChatGPT; respeite `snapshot_age_min` (snapshot antigo = valor mínimo, não exato) |
| OpenRouter (só decisões do Jev) | `tools/agents/quota-check.sh` → `openrouter`: `usage_usd`, `limit_usd`, `limit_remaining_usd`, `usage_daily_usd` |

Use sempre a **maior** porcentagem entre a janela de 5 h e a semanal.

### Quando auditar

- No início de toda sessão (junto com a leitura de `docs/STATUS.md`).
- Antes de disparar qualquer lote de subagentes.
- Ao fim de cada tarefa grande e antes de encerrar a sessão.

### Regras de decisão

| Situação | Ação |
|---|---|
| Claude < 75% | Claude trabalha normalmente; delega o que for paralelizável |
| Claude com menos de 20% livre | Escolhas da Anthropic feitas pelo Jev vão para o Codex, não para o Claude |
| Claude ≥ 75% | Claude passa a só supervisionar: escreve o handoff em `docs/STATUS.md`, faz commit e delega a próxima tarefa |
| OpenRouter com saldo < US$ 0,10 | Subagentes continuam, mas no modelo padrão (`JEV_FALLBACK_MODEL`), sem decisão do Jev; avisar o usuário |
| Codex (ChatGPT) ≥ 90% ou limite atingido | Usar opencode + DeepSeek |
| Todos os executores esgotados | Registrar estado e próximos passos em `docs/STATUS.md`, commit e push, e **agendar a retomada** para o menor `resetsAt` + 2 min (tarefa agendada única) |
| Claude chegou a 95% durante uma tarefa | Parar em ponto seguro, commit, push, handoff e agendar a retomada |

**Regra de ouro**: nenhuma sessão termina sem (a) o estado commitado e enviado ao remoto e
(b) um próximo passo claro em `docs/STATUS.md` ou uma retomada agendada. Projeto parado sem
retomada agendada é bug de processo.

### Registro

Cada auditoria relevante (troca de executor, pausa, retomada agendada) é anotada em
`docs/STATUS.md` → seção "Agora", com os números lidos e o horário de retomada.

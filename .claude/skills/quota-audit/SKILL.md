---
name: quota-audit
description: Audit Claude, Codex and OpenRouter quotas and decide who works next so the project never freezes. Use at session start, before launching subagents, at the end of big tasks, for "/quota-audit", "verifica as cotas", or when a limit message appears.
---

# Quota audit

Policy and thresholds: `docs/process/agentes-e-cotas.md` §3. This skill is the procedure.

1. **Claude**: call `mcp__ccd_session_mgmt__get_usage` (load it with ToolSearch if deferred).
   Take the higher `percentUsed` of the 5-hour and weekly windows and its `resetsAt`.
2. **Codex and OpenRouter**: run `tools/agents/quota-check.sh`. For Codex, treat a snapshot with a
   large `snapshot_age_min` as a lower bound; `reset_passed: true` means that window is free again.
3. **Decide** with the table in §3 of the policy doc: who works next (Claude, Jev subagents, Codex via
   ChatGPT, opencode + DeepSeek), or pause.
4. **If everything is exhausted** or Claude is at 95%+: commit, push, update `docs/STATUS.md`
   ("Agora": numbers read, next step, resume time) and create a one-time scheduled task at the
   earliest `resetsAt` + 2 minutes that resumes from `docs/STATUS.md`
   (`mcp__scheduled-tasks__create_scheduled_task`).
5. **Report in Portuguese**, one line per quota: `Claude 39% (5h) / 67% (semana)`,
   `Codex 31% (semana, snapshot de 4 dias)`, `OpenRouter US$ 3,70 restantes de US$ 5`, then the decision.

Never print API keys. Never start a subagent batch whose estimated cost exceeds the OpenRouter balance.

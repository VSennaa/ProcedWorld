---
name: jev-subagents
description: Delegate tasks to Codex (ChatGPT) subagents whose model is chosen per task by Jev Router on OpenRouter. Claude writes the prompts, launches, waits, reviews and commits. Use for "/jev-subagents", "usa sub agents", "delega pro codex com o Jev", or parallel drafting/implementation of independent files.
---

# Jev subagents (Claude supervises)

Tool: `tools/agents/jev-codex.sh <task-name> <prompt-file> [read-only|workspace-write]`.
OpenRouter is used ONLY for Jev's decision (`typesafe/jev-router`, `max_tokens: 1`, choice in the
response `model`; ~US$ 0.00001-0.002). The pick is mapped to a Codex model (exact `openai/*` match,
else nearest output price in log scale among `JEV_CODEX_MODELS`) and Codex runs the task on the
**ChatGPT login**. Never run task execution through OpenRouter. Low credit or router error →
`JEV_FALLBACK_MODEL` (gpt-6-sol). Decisions go to `.agent-runs/decisions.jsonl` (gitignored), also
early calibration data for the T1 layer (ADR-0005). Policy: `docs/process/agentes-e-cotas.md`.

Why one model per task: letting Jev Router route every request of a Codex session breaks on the
second request ("No models satisfy the decisions policy").

## Requirements
- `OPENROUTER_API_KEY` in the environment (decision only; never print it, never put it in a file).
- `codex` on PATH (or `CODEX_BIN`), logged in with ChatGPT; its quota is what execution spends. Codex reads `AGENTS.md` (UTF-8 rule, no git commands).

## Procedure
1. **Split** the work into independent tasks, each owning distinct files. Never let two subagents
   write the same file.
2. **Commit** the current state first so subagent edits can be reviewed and reverted with git.
3. **Write one prompt file per task** in the scratchpad: context files to read, the exact file(s) to
   write, acceptance criteria, what not to touch. Keep project rules (CLAUDE.md §8) explicit.
   Name only the files the task really needs to read: "read every pillar" pushed one run to 412k tokens.
4. **Audit first** (skill `quota-audit`), then **launch** in the background with `run_in_background`,
   **at most 2 in parallel** (keeps Codex rate limits and review load manageable):
   `ls prompts/*.md | xargs -P 2 -I{} sh -c 'tools/agents/jev-codex.sh "$(basename {} .md)" {}'`
   Do not edit the files they own while they run. Do not poll; wait for the exit notification.
5. **Review** every result: `git diff`, check encoding (no `�`), consistency with ADRs and other
   files, no scope creep. Fix or rerun; subagent output is a draft, not a decision.
6. **Commit** on the proper branch and report (in Portuguese): Jev's pick and the Codex model per task,
   tokens, failures, what still needs the user's approval.

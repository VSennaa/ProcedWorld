---
name: jev-subagents
description: Delegate tasks to Codex (ChatGPT) subagents whose model is chosen per task by Jev Router on OpenRouter. Claude writes the prompts, launches, waits, reviews and commits. Use for "/jev-subagents", "usa sub agents", "delega pro codex com o Jev", or parallel drafting/implementation of independent files.
---

# Jev subagents (Claude supervises)

Tool: `tools/agents/jev-codex.sh <task-name> <prompt-file> [read-only|workspace-write]`.
It asks `typesafe/jev-router` (OpenRouter) to pick a model for the task (`max_tokens: 1`, the choice
comes in the response `model`), checks the model supports tools, falls back to
`JEV_FALLBACK_MODEL` (default `deepseek/deepseek-v4-flash`), then runs `codex exec` on that model
through OpenRouter. Every decision is appended to `.agent-runs/decisions.jsonl` (gitignored): it is
also early calibration data for the T1 layer (ADR-0005).

Why one model per task: letting Jev Router route every request of a Codex session breaks on the
second request ("No models satisfy the decisions policy"), because the conversation carries
model-specific reasoning items.

## Requirements
- `OPENROUTER_API_KEY` in the environment (never print it, never put it in a file).
- `codex` on PATH (or `CODEX_BIN`). Codex reads `AGENTS.md` (UTF-8 rule, no git commands).

## Procedure
1. **Split** the work into independent tasks, each owning distinct files. Never let two subagents
   write the same file.
2. **Commit** the current state first so subagent edits can be reviewed and reverted with git.
3. **Write one prompt file per task** in the scratchpad: context files to read, the exact file(s) to
   write, acceptance criteria, what not to touch. Keep project rules (CLAUDE.md §8) explicit.
4. **Launch** in the background with `run_in_background`, at most 4 in parallel:
   `ls prompts/*.md | xargs -P 4 -I{} sh -c 'tools/agents/jev-codex.sh "$(basename {} .md)" {}'`
   Do not edit the files they own while they run. Do not poll; wait for the exit notification.
5. **Review** every result: `git diff`, check encoding (no `�`), consistency with ADRs and other
   files, no scope creep. Fix or rerun; subagent output is a draft, not a decision.
6. **Commit** on the proper branch and report (in Portuguese): which model Jev picked per task,
   tokens, failures, what still needs the user's approval.

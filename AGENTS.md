# AGENTS.md

Instructions for non-Claude agents (Codex, opencode, etc.) working in this repository.

1. **Read `CLAUDE.md` first.** It is the project constitution and applies to every agent. Then read
   `docs/STATUS.md`.
2. **Encoding: all files are UTF-8 (docs are in Brazilian Portuguese with accents).**
   On Windows PowerShell 5.1 always pass the encoding explicitly:
   `Get-Content -Encoding utf8 <file>` and `Set-Content -Encoding utf8` / `Out-File -Encoding utf8`.
   Never let a tool rewrite a file in ANSI/cp1252. Line endings: LF.
3. **Read your brief in `docs/briefs/` if one is named. Only touch the files your task names.** Do not reorganize, rename or "clean up" anything else.
4. **Do not run `git commit`, `git push`, or change branches.** The supervising agent (Claude) reviews
   and commits.
5. Never write secrets, API keys, IPs or hostnames into files.
6. When your task is done, end with a short summary: files changed, what is a proposal vs. decided,
   and open questions.

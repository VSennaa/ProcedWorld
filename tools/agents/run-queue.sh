#!/usr/bin/env bash
# Runs subagent tasks ONE AT A TIME (parallel Codex runs on Windows have failed to write files).
# Meant to be launched detached, so it is not killed by the 10-minute background limit and does not
# wake the supervisor (saves Claude quota). The supervisor reads the summary afterwards.
#
# Usage: tools/agents/run-queue.sh <queue-file>
# Queue line: name|brief path (relative to repo)|working dir (absolute)|search (0/1)
# Output: .agent-runs/queue-<queue basename>.log (one line per task + final DONE line)
set -uo pipefail
queue=${1:?queue file}
root=$(git rev-parse --show-toplevel)
log="$root/.agent-runs/queue-$(basename "$queue" .txt).log"
mkdir -p "$root/.agent-runs"
runs="$root/.agent-runs"
board() { python "$root/tools/board/update.py" "$@" >/dev/null 2>&1 || true; }
title_of() { head -1 "$root/$1" | sed 's/^# *//; s/^Brief *//'; }
model_of() { grep "\"task\": \"$1\"" "$runs/decisions.jsonl" 2>/dev/null | tail -1 | sed -n 's/.*"model": "\([^"]*\)".*/codex · /p'; }
seed_board() { while IFS='|' read -r n b _rest; do [ -z "$n" ] || [ "${n:0:1}" = "#" ] && continue; board "$n" todo --title "$(title_of "$b")"; done < "$queue"; }
echo "START $(date '+%F %T')" >> "$log"
seed_board
while IFS='|' read -r name brief dir search; do
  [ -z "$name" ] || [ "${name:0:1}" = "#" ] && continue
  start=$(date '+%T')
  board "$name" doing --branch "$(cd "$dir" && git branch --show-current)"
  out=$(cd "$dir" && JEV_CODEX_SEARCH="$search" "$root/tools/agents/jev-codex.sh" "$name" "$root/$brief" workspace-write < /dev/null 2>&1 | tail -1)
  enc=$(cd "$dir" && python "$root/tools/agents/check-encoding.py" 2>&1 | tail -1)
  if [[ "$out" == *"exit 0"* ]]; then board "$name" done --agent "$(model_of "$name")" --note "$enc"; else board "$name" failed --agent "$(model_of "$name")" --note "$out"; fi
  echo "$name | $start-$(date '+%T') | $out | $enc" >> "$log"
done < "$queue"
echo "DONE $(date '+%F %T')" >> "$log"

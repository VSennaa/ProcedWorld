#!/usr/bin/env bash
# Build loop for code tasks: Codex writes, the VPS compiles and tests, failures go back to Codex.
# Runs tasks ONE AT A TIME; meant to be launched detached (see run-queue.sh for docs tasks).
#
# Usage: tools/agents/build-loop.sh <queue-file>
# Queue line: name|brief path (relative to repo)|working dir (absolute)|max fix attempts
# Output: .agent-runs/build-<queue basename>.log, plus <name>.test-N.txt with each test run.
set -uo pipefail
queue=${1:?queue file}
root=$(git rev-parse --show-toplevel)
runs="$root/.agent-runs"; mkdir -p "$runs"
log="$runs/build-$(basename "$queue" .txt).log"
echo "START $(date '+%F %T')" >> "$log"

while IFS='|' read -r name brief dir attempts; do
  [ -z "$name" ] || [ "${name:0:1}" = "#" ] && continue
  attempts=${attempts:-3}
  start=$(date '+%T')
  (cd "$dir" && "$root/tools/agents/jev-codex.sh" "$name" "$root/$brief" workspace-write < /dev/null > /dev/null 2>&1)
  status="FAIL"
  for n in $(seq 0 "$attempts"); do
    out="$runs/$name.test-$n.txt"
    "$root/tools/dev/vps-test.sh" "$dir" > "$out" 2>&1
    if grep -q "VPS_EXIT=0" "$out"; then status="PASS(fixes=$n)"; break; fi
    [ "$n" -ge "$attempts" ] && break
    fix="$runs/$name.fix-$((n+1)).md"
    {
      echo "# Correção automática $((n+1)) da tarefa $name"
      echo
      echo "O brief original é \`$brief\`; releia-o. O código **não passou** em \`cargo test --workspace\`"
      echo "rodado na VPS (Linux, Docker rust:1-slim). Corrija os erros abaixo sem mudar o escopo da tarefa."
      echo "Não rode cargo localmente (o Windows bloqueia executáveis compilados e não há rede)."
      echo "Grave só com apply_patch. Não rode git. Não desative testes para fazê-los passar."
      echo
      echo '```text'
      grep -v '^\s*Compiling\|^\s*Downloaded\|^\s*Downloading' "$out" | tail -150
      echo '```'
    } > "$fix"
    (cd "$dir" && "$root/tools/agents/jev-codex.sh" "$name-fix$((n+1))" "$fix" workspace-write < /dev/null > /dev/null 2>&1)
  done
  summary=$(grep -E "^test result|warning: unused" "$runs/$name".test-*.txt 2>/dev/null | tail -2 | tr '\n' ' ')
  if [[ "$status" == PASS* ]] && [ "${BUILD_LOOP_COMMIT:-1}" = "1" ]; then
    # Commit each passing task before the next one starts, so commits never mix tasks.
    (cd "$dir" && git add engine && git commit -q -m "feat(engine): $name passes cargo test on the VPS (build loop)

Brief: $brief

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>" && git push -q) >/dev/null 2>&1
  fi
  echo "$name | $start-$(date '+%T') | $status | $summary" >> "$log"
done < "$queue"
echo "DONE $(date '+%F %T')" >> "$log"

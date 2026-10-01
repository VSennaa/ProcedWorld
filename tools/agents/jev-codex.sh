#!/usr/bin/env bash
# Delegate one task to a Codex subagent. Jev Router (OpenRouter) picks the model once per task;
# Codex then runs the whole task on that model (Jev cannot switch models mid-session: the second
# request of an agent loop fails with "No models satisfy the decisions policy").
#
# Usage: tools/agents/jev-codex.sh <task-name> <prompt-file> [sandbox]
#   sandbox: read-only | workspace-write (default)
# Env:   OPENROUTER_API_KEY (required), CODEX_BIN, JEV_FALLBACK_MODEL
# Output: .agent-runs/<task-name>.{log,out.txt}, decisions appended to .agent-runs/decisions.jsonl
set -euo pipefail

name=${1:?task name}; prompt_file=${2:?prompt file}; sandbox=${3:-workspace-write}
: "${OPENROUTER_API_KEY:?OPENROUTER_API_KEY not set}"
root=$(git rev-parse --show-toplevel)
runs="$root/.agent-runs"; mkdir -p "$runs"
codex_bin=${CODEX_BIN:-codex}
fallback=${JEV_FALLBACK_MODEL:-deepseek/deepseek-v4-flash}
py=$(command -v python3 || command -v python)
min_credit=${JEV_MIN_CREDIT:-1.0}

# 0. Credit preflight: OpenRouter reserves worst-case cost of in-flight requests and answers 402
#    even with balance left, so refuse to start below a safety margin (exit 75 = try again later).
remaining=$("$root/tools/agents/quota-check.sh" | "$py" -c 'import json,sys; o=json.load(sys.stdin)["openrouter"]; r=o.get("limit_remaining_usd"); print("" if r is None else r)')
if [ -n "$remaining" ] && "$py" -c "import sys; sys.exit(0 if float('$remaining') < float('$min_credit') else 1)"; then
  echo "[$name] OpenRouter credit US\$ $remaining below JEV_MIN_CREDIT=$min_credit; not starting" >&2
  exit 75
fi

# 1. Jev picks a model. max_tokens=1: we only want the routing decision, reported in `model`.
pick=$("$py" - "$prompt_file" "$fallback" <<'PY'
import json, os, sys, urllib.request
prompt = open(sys.argv[1], encoding="utf-8").read()
fallback = sys.argv[2]
def call(url, body=None):
    req = urllib.request.Request(url, data=json.dumps(body).encode() if body else None,
        headers={"Authorization": "Bearer " + os.environ["OPENROUTER_API_KEY"],
                 "Content-Type": "application/json"})
    return json.load(urllib.request.urlopen(req, timeout=60))
out = {"model": fallback, "decision_cost": 0, "fallback": True, "reason": ""}
try:
    r = call("https://openrouter.ai/api/v1/chat/completions", {
        "model": "typesafe/jev-router", "max_tokens": 1, "usage": {"include": True},
        "messages": [
            {"role": "system", "content": "You are an autonomous coding and writing agent with shell "
             "tools, working on a multi-step task in a repository."},
            {"role": "user", "content": prompt[:6000]}]})
    chosen = r["model"]
    models = call("https://openrouter.ai/api/v1/models")["data"]
    tools_ok = any(m["id"] == chosen and "tools" in m.get("supported_parameters", []) for m in models)
    out.update(decision_cost=r.get("usage", {}).get("cost", 0))
    if tools_ok:
        out.update(model=chosen, fallback=False)
    else:
        out["reason"] = f"jev chose {chosen} without tool support"
except Exception as e:
    out["reason"] = f"jev error: {e}"
print(json.dumps(out))
PY
)
model=$("$py" -c 'import json,sys; print(json.loads(sys.argv[1])["model"])' "$pick")
echo "[$name] model: $model ($pick)"

# 2. Codex runs the task on the chosen model through OpenRouter.
start=$(date +%s); status=0
"$codex_bin" exec -C "$root" -s "$sandbox" \
  -c model_provider=openrouter \
  -c 'model_providers.openrouter.name="OpenRouter"' \
  -c 'model_providers.openrouter.base_url="https://openrouter.ai/api/v1"' \
  -c 'model_providers.openrouter.env_key="OPENROUTER_API_KEY"' \
  -m "$model" -o "$runs/$name.out.txt" - < "$prompt_file" > "$runs/$name.log" 2>&1 || status=$?
tokens=$(grep -A1 '^tokens used' "$runs/$name.log" | tail -1 | tr -dc '0-9' || true)

"$py" - "$name" "$pick" "$status" "${tokens:-0}" "$(( $(date +%s) - start ))" >> "$runs/decisions.jsonl" <<'PY'
import json, sys, datetime
name, pick, status, tokens, secs = sys.argv[1:]
rec = {"ts": datetime.datetime.now(datetime.timezone.utc).isoformat(timespec="seconds"),
       "task": name, **json.loads(pick), "exit": int(status), "tokens": int(tokens or 0), "seconds": int(secs)}
print(json.dumps(rec))
PY
echo "[$name] exit $status, tokens ${tokens:-?}"
exit "$status"

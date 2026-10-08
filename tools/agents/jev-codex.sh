#!/usr/bin/env bash
# Delegate one task to a Codex subagent running on the ChatGPT login. Jev Router (OpenRouter) only
# DECIDES the model; OpenRouter credit is spent on that decision alone (~US$ 0.0002-0.002), never on
# executing the task.
#
# Jev Router chooses among all OpenRouter models and cannot be restricted, so its pick is mapped to a
# Codex model by capability, using tools/agents/model-scores.json (Fabio Akita's LLM benchmark):
#   the CHEAPEST Codex model whose score >= the score of Jev's pick.
# If Jev's pick is not in the benchmark, its score is estimated from the benchmarked model with the
# closest OpenRouter output price (log scale). Exact match when Jev picks a Codex model directly.
#
# Usage: tools/agents/jev-codex.sh <task-name> <prompt-file> [sandbox]
#   sandbox: read-only | workspace-write (default)
# Env:   OPENROUTER_API_KEY (required, decision only), CODEX_BIN, JEV_CODEX_MODELS, JEV_FALLBACK_MODEL,
#        JEV_MIN_CREDIT, JEV_CLAUDE_OK, JEV_CODEX_SEARCH (=1 enables Codex live web search)
# Anthropic picks: when the supervisor (Claude) has quota to spare it sets JEV_CLAUDE_OK=1; then an
# `anthropic/*` pick is NOT run on Codex: the script records it and exits 76 so Claude runs the task
# itself (Agent tool). Without JEV_CLAUDE_OK the pick is mapped to Codex as usual.
# Executor (JEV_EXECUTOR, default opencode since 2026-10-07): `opencode` runs the task with opencode
# on the DeepSeek API (key DEEPSEEK_API_KEY, prepaid; provider in ./opencode.json), mapping Jev's pick to
# the cheapest model in JEV_OPENCODE_MODELS whose benchmark score is >= the pick's (else the best one);
# `codex` runs it on the ChatGPT login as before.
# Output: .agent-runs/<task-name>.{log,out.txt}; decisions appended to .agent-runs/decisions.jsonl
set -euo pipefail

name=${1:?task name}; prompt_file=${2:?prompt file}; sandbox=${3:-workspace-write}
: "${OPENROUTER_API_KEY:?OPENROUTER_API_KEY not set}"
root=$(git rev-parse --show-toplevel)
runs="$root/.agent-runs"; mkdir -p "$runs"
codex_bin=${CODEX_BIN:-codex}
export JEV_CODEX_MODELS=${JEV_CODEX_MODELS:-gpt-6-astra,gpt-6-sol,gpt-6-luna,gpt-5.6-sol,gpt-5.6-terra,gpt-5.6-luna,gpt-5.5}
export JEV_SCORES="$root/tools/agents/model-scores.json"
export JEV_FALLBACK_MODEL=${JEV_FALLBACK_MODEL:-gpt-6-sol}
min_credit=${JEV_MIN_CREDIT:-0.10}
py=$(command -v python3 || command -v python)

# 0. Credit preflight for the decision call (exit 75 = try again later). If it fails, Codex still
#    runs on the fallback model: the project must not stop because of the router.
remaining=$("$root/tools/agents/quota-check.sh" | "$py" -c 'import json,sys; o=json.load(sys.stdin)["openrouter"]; r=o.get("limit_remaining_usd"); print("" if r is None else r)')
skip_jev=0
if [ -n "$remaining" ] && "$py" -c "import sys; sys.exit(0 if float('$remaining') < float('$min_credit') else 1)"; then
  echo "[$name] OpenRouter credit US\$ $remaining below JEV_MIN_CREDIT=$min_credit; using fallback model" >&2
  skip_jev=1
fi

# 1. Jev decides (max_tokens=1; the choice comes back in `model`), then map it to a Codex model.
pick=$(SKIP_JEV=$skip_jev "$py" - "$prompt_file" <<'PY'
import json, math, os, sys, urllib.request
prompt = open(sys.argv[1], encoding="utf-8").read()
codex_models = os.environ["JEV_CODEX_MODELS"].split(",")
out = {"jev_choice": None, "model": os.environ["JEV_FALLBACK_MODEL"], "mapping": "fallback",
       "decision_cost": 0, "reason": ""}
def call(url, body=None):
    req = urllib.request.Request(url, data=json.dumps(body).encode() if body else None,
        headers={"Authorization": "Bearer " + os.environ["OPENROUTER_API_KEY"],
                 "Content-Type": "application/json"})
    return json.load(urllib.request.urlopen(req, timeout=60))
if os.environ.get("SKIP_JEV") == "1":
    out["reason"] = "low OpenRouter credit"
else:
    try:
        r = call("https://openrouter.ai/api/v1/chat/completions", {
            "model": "typesafe/jev-router", "max_tokens": 1, "usage": {"include": True},
            "messages": [
                {"role": "system", "content": "You are an autonomous coding and writing agent with shell "
                 "tools, working on a multi-step task in a repository."},
                # The task head is enough to judge difficulty and keeps the decision cheap.
                {"role": "user", "content": prompt[:2000]}]})
        chosen = r["model"]
        out.update(jev_choice=chosen, decision_cost=r.get("usage", {}).get("cost", 0))
        slug = chosen.split("/", 1)[-1]
        scores = json.load(open(os.environ["JEV_SCORES"], encoding="utf-8"))["scores"]
        if chosen.startswith("openai/") and slug in codex_models:
            out.update(model=slug, mapping="exact")
        else:
            mapping = "benchmark"
            target = scores.get(chosen, {}).get("score")
            if target is None:
                # Not benchmarked: borrow the score of the benchmarked model priced closest to it.
                price = {m["id"]: float(m["pricing"]["completion"])
                         for m in call("https://openrouter.ai/api/v1/models")["data"]
                         if float(m["pricing"].get("completion") or -1) > 0}
                known = [k for k in scores if k in price]
                if chosen in price and known:
                    near = min(known, key=lambda k: abs(math.log(price[k]) - math.log(price[chosen])))
                    target, mapping = scores[near]["score"], f"benchmark-estimated-from:{near}"
            codex = {c: scores["openai/" + c] for c in codex_models if "openai/" + c in scores}
            ok = [c for c in codex if codex[c]["score"] >= (target or 0)]
            if target is not None and ok:
                best = min(ok, key=lambda c: (codex[c]["cost_usd"], -codex[c]["score"]))
                out.update(model=best, mapping=mapping, jev_choice_score=target,
                           codex_score=codex[best]["score"])
            elif target is not None and codex:
                best = max(codex, key=lambda c: (codex[c]["score"], -codex[c]["cost_usd"]))
                out.update(model=best, mapping=mapping + ":best-available", jev_choice_score=target,
                           codex_score=codex[best]["score"])
            else:
                out["reason"] = f"cannot score {chosen}"
    except Exception as e:
        out["reason"] = f"jev error: {e}"
print(json.dumps(out))
PY
)
model=$("$py" -c 'import json,sys; print(json.loads(sys.argv[1])["model"])' "$pick")
jev_choice=$("$py" -c 'import json,sys; print(json.loads(sys.argv[1]).get("jev_choice") or "")' "$pick")

if [ "${JEV_CLAUDE_OK:-0}" = "1" ] && [[ "$jev_choice" == anthropic/* ]]; then
  "$py" - "$name" "$pick" >> "$runs/decisions.jsonl" <<'PY2'
import json, sys, datetime
rec = {"ts": datetime.datetime.now(datetime.timezone.utc).isoformat(timespec="seconds"),
       "task": sys.argv[1], **json.loads(sys.argv[2]), "executor": "claude", "exit": 76}
print(json.dumps(rec))
PY2
  echo "[$name] executor: claude ($jev_choice) - run it with the Agent tool"
  exit 76
fi
executor=${JEV_EXECUTOR:-opencode}
start=$(date +%s); status=0
if [ "$executor" = "opencode" ]; then
  # 2a. opencode on the DeepSeek API: map Jev's pick by benchmark score (Akita) to a DeepSeek model.
  : "${DEEPSEEK_API_KEY:?DEEPSEEK_API_KEY not set (needed by JEV_EXECUTOR=opencode)}"
  export JEV_OPENCODE_MODELS=${JEV_OPENCODE_MODELS:-deepseek/deepseek-flash=deepseek/deepseek-v4.1-flash,deepseek/deepseek-v4-pro=deepseek/deepseek-v4-pro-0813}
  model=$("$py" - "$pick" <<'PY3'
import json, os, sys
pick = json.loads(sys.argv[1])
scores = json.load(open(os.environ["JEV_SCORES"], encoding="utf-8"))["scores"]
pairs = [item.split("=") for item in os.environ["JEV_OPENCODE_MODELS"].split(",")]
ranked = [(scores.get(bench, {}).get("score", 0), scores.get(bench, {}).get("cost_usd", 1e9), oc) for oc, bench in pairs]
need = pick.get("jev_choice_score") or scores.get(pick.get("jev_choice") or "", {}).get("score") or 0
ok = [r for r in ranked if r[0] >= need]
print(min(ok, key=lambda r: r[1])[2] if ok else max(ranked)[2])
PY3
)
  pick=$("$py" -c 'import json,sys; d=json.loads(sys.argv[1]); d["model"]=sys.argv[2]; d["mapping"]=d.get("mapping","")+"+opencode"; print(json.dumps(d))' "$pick" "$model")
  echo "[$name] opencode model: $model ($pick)"
  agent=build; [ "$sandbox" = "read-only" ] && agent=plan
  (cd "$root" && opencode run --standalone --auto --agent "$agent" -m "$model" -f "$prompt_file" \
    "Siga exatamente as instruções do arquivo anexo. Responda em português ao final.") \
    > "$runs/$name.log" 2>&1 || status=$?
  # The final answer is the text after the last tool line; keep the whole log as out.txt for review.
  cp "$runs/$name.log" "$runs/$name.out.txt"
  tokens=0
else
  echo "[$name] codex model: $model ($pick)"
  # 2b. Codex runs the task on the ChatGPT login (default provider), on the chosen model.
  search_flag=(); [ "${JEV_CODEX_SEARCH:-0}" = "1" ] && search_flag=(--search)
  "$codex_bin" "${search_flag[@]}" exec -C "$root" -s "$sandbox" -m "$model" \
    -o "$runs/$name.out.txt" - < "$prompt_file" > "$runs/$name.log" 2>&1 || status=$?
  tokens=$(grep -A1 '^tokens used' "$runs/$name.log" | tail -1 | tr -dc '0-9' || true)
fi

[ "$executor" = "opencode" ] && export JEV_EXECUTOR_USED=opencode-deepseek
"$py" - "$name" "$pick" "$status" "${tokens:-0}" "$(( $(date +%s) - start ))" >> "$runs/decisions.jsonl" <<'PY'
import json, os, sys, datetime
name, pick, status, tokens, secs = sys.argv[1:]
rec = {"ts": datetime.datetime.now(datetime.timezone.utc).isoformat(timespec="seconds"),
       "task": name, **json.loads(pick), "executor": os.environ.get("JEV_EXECUTOR_USED", "codex-chatgpt"), "exit": int(status),
       "tokens": int(tokens or 0), "seconds": int(secs)}
print(json.dumps(rec))
PY
echo "[$name] exit $status, tokens ${tokens:-?}"
exit "$status"

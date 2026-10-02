#!/usr/bin/env bash
# Quota audit for external executors. Prints one JSON object:
#   codex:      latest rate-limit snapshot from ~/.codex/sessions (ChatGPT-backed runs only)
#   openrouter: credit usage/limit of OPENROUTER_API_KEY (used by Jev-routed subagents)
# Claude's own quota is NOT here: it comes from the desktop tool `get_usage`
# (mcp__ccd_session_mgmt__get_usage). See docs/process/agentes-e-cotas.md.
set -euo pipefail
py=$(command -v python3 || command -v python)
"$py" - <<'PY'
import glob, json, os, re, time, urllib.request

now = time.time()
out = {"checked_at": int(now)}

# Codex: newest session file that carries a non-null primary window.
codex = {"available": False}
pat = re.compile(r'"rate_limits":\{"limit_id":"codex".*?"primary":(\{[^}]*\}),"secondary":(\{[^}]*\}|null)')
files = sorted(glob.glob(os.path.expanduser("~/.codex/sessions/*/*/*/*.jsonl")),
               key=os.path.getmtime, reverse=True)[:300]
for f in files:
    with open(f, encoding="utf-8", errors="replace") as fh:
        hits = pat.findall(fh.read())
    if hits:
        prim, sec = (json.loads(x) if x != "null" else None for x in hits[-1])
        def win(w):
            if not w: return None
            reset = w.get("resets_at") or 0
            passed = reset and reset <= now
            return {"used_percent": 0.0 if passed else w.get("used_percent"),
                    "window_minutes": w.get("window_minutes"),
                    "resets_at": reset, "reset_passed": bool(passed)}
        codex = {"available": True, "snapshot_age_min": int((now - os.path.getmtime(f)) / 60),
                 "five_hour": win(prim), "weekly": win(sec)}
        used = [w["used_percent"] for w in (codex["five_hour"], codex["weekly"]) if w]
        codex["max_used_percent"] = max(used) if used else None
        break
out["codex"] = codex

# OpenRouter: /api/v1/key returns usage and limit for the key.
orr = {"available": False}
key = os.environ.get("OPENROUTER_API_KEY")
if key:
    try:
        req = urllib.request.Request("https://openrouter.ai/api/v1/key",
                                     headers={"Authorization": "Bearer " + key})
        d = json.load(urllib.request.urlopen(req, timeout=30))["data"]
        orr = {"available": True, "usage_usd": d.get("usage"), "limit_usd": d.get("limit"),
               "limit_remaining_usd": d.get("limit_remaining"),
               "usage_daily_usd": d.get("usage_daily")}
    except Exception as e:
        orr = {"available": False, "error": str(e)}
out["openrouter"] = orr
print(json.dumps(out, indent=1))
PY

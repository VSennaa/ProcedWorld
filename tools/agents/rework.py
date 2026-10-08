"""Rework log: one JSON line per delivered task, and a comparison report per executor.

Usage:
  python tools/agents/rework.py add '<json object>'    # append one entry (fields below)
  python tools/agents/rework.py report                 # rewrite docs/process/retrabalho.md

Entry fields (missing = unknown):
  date, task, area, critical (bool), executor (codex | opencode | claude-sonnet | claude-opus),
  model, judge: [{"by": model, "score": n, "verdict": "APROVADO|APROVADO_COM_RESSALVAS|REPROVADO",
  "blockers": n}], auto_fixes (build-loop fix runs), agent_fixes (extra runs of an agent to fix
  findings), supervisor_fixes (fixes Claude made by hand), handoffs (task moved to another executor),
  tokens, cost_usd, minutes, outcome (merged | parked | failed), notes.
"""
import io, json, os, sys
from collections import defaultdict

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
LOG = os.path.join(ROOT, "docs", "process", "retrabalho.jsonl")
DOC = os.path.join(ROOT, "docs", "process", "retrabalho.md")


def load():
    if not os.path.exists(LOG):
        return []
    with io.open(LOG, encoding="utf-8") as f:
        return [json.loads(line) for line in f if line.strip()]


def rework_units(e):
    """Runs that redid work: judge rejections + automatic fixes + agent fixes + supervisor fixes + handoffs."""
    rejected = sum(1 for j in e.get("judge", []) if j.get("verdict") == "REPROVADO")
    return rejected + e.get("auto_fixes", 0) + e.get("agent_fixes", 0) + e.get("supervisor_fixes", 0) + e.get("handoffs", 0)


def report():
    rows = load()
    by = defaultdict(list)
    for e in rows:
        by[e["executor"]].append(e)
    out = ["# Retrabalho por executor",
           "",
           "> Gerado por `python tools/agents/rework.py report` a partir de `retrabalho.jsonl`. Não editar à mão.",
           "",
           "Retrabalho = reprovações do juiz + correções automáticas (loop de build) + execuções extras do agente"
           " para corrigir achados + correções feitas à mão pelo supervisor + trocas de executor.",
           "",
           "| Executor | Tarefas | Mescladas | Retrabalho médio | 1ª nota média do juiz | Nota final média | Correções do supervisor | Trocas de executor | Custo direto |",
           "|---|---|---|---|---|---|---|---|---|"]
    for ex in sorted(by):
        es = by[ex]
        merged = sum(1 for e in es if e.get("outcome") == "merged")
        firsts = [e["judge"][0]["score"] for e in es if e.get("judge")]
        finals = [e["judge"][-1]["score"] for e in es if e.get("judge")]
        cost = [e["cost_usd"] for e in es if e.get("cost_usd") is not None]
        avg = lambda xs: f"{sum(xs) / len(xs):.1f}" if xs else "—"
        out.append(f"| {ex} | {len(es)} | {merged} | {avg([rework_units(e) for e in es])} | {avg(firsts)} | {avg(finals)} | "
                   f"{sum(e.get('supervisor_fixes', 0) for e in es)} | {sum(e.get('handoffs', 0) for e in es)} | "
                   f"{('US$ %.2f' % sum(cost)) if cost else 'cota da assinatura'} |")
    out += ["", "## Tarefas", "",
            "| Data | Tarefa | Executor / modelo | Crítica | Juiz (notas) | Retrabalho | Resultado | Observações |",
            "|---|---|---|---|---|---|---|---|"]
    for e in rows:
        judge = " → ".join(f"{j.get('score', '?')}{'✗' if j.get('verdict') == 'REPROVADO' else ''}" for j in e.get("judge", [])) or "—"
        out.append(f"| {e.get('date', '')} | {e['task']} | {e['executor']} / {e.get('model', '?')} | {'sim' if e.get('critical') else 'não'} | "
                   f"{judge} | {rework_units(e)} | {e.get('outcome', '?')} | {e.get('notes', '')} |")
    out += ["", "✗ = reprovado. Custo direto: só o que é cobrado por uso (DeepSeek pré-pago); Codex e Claude gastam cota da assinatura (janela de 5 h e semanal)."]
    with io.open(DOC, "w", encoding="utf-8", newline="\n") as f:
        f.write("\n".join(out) + "\n")
    print(f"wrote {DOC} ({len(rows)} entries)")


def main(argv):
    if argv[:1] == ["add"] and len(argv) == 2:
        entry = json.loads(argv[1])
        for key in ("task", "executor"):
            if key not in entry:
                sys.exit(f"missing field: {key}")
        os.makedirs(os.path.dirname(LOG), exist_ok=True)
        with io.open(LOG, "a", encoding="utf-8", newline="\n") as f:
            f.write(json.dumps(entry, ensure_ascii=False) + "\n")
        return 0
    if argv[:1] == ["report"]:
        report()
        return 0
    print(__doc__)
    return 2


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))

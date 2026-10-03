"""Kanban board for subagent tasks. One cheap shell call per update; no tokens for the agents.

Usage:
  python tools/board/update.py <id> <status> [--title T] [--agent A] [--branch B] [--note N]
  python tools/board/update.py --clear-done      # drop finished cards
status: todo | doing | fix | done | failed

State lives in .agent-runs/board.json (gitignored). Each call regenerates .agent-runs/board-data.js
and copies the page to .agent-runs/board.html, which works from file:// (open it in a browser).
"""
import json, os, shutil, sys, datetime

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
RUNS = os.path.join(ROOT, ".agent-runs")
STATE = os.path.join(RUNS, "board.json")
STATUSES = ("todo", "doing", "fix", "done", "failed")


def load():
    try:
        with open(STATE, encoding="utf-8") as f:
            return json.load(f)
    except (OSError, ValueError):
        return {"cards": {}}


def save(board):
    os.makedirs(RUNS, exist_ok=True)
    board["updated"] = datetime.datetime.now().strftime("%Y-%m-%d %H:%M:%S")
    with open(STATE, "w", encoding="utf-8") as f:
        json.dump(board, f, ensure_ascii=False, indent=1)
    with open(os.path.join(RUNS, "board-data.js"), "w", encoding="utf-8") as f:
        f.write("window.BOARD = " + json.dumps(board, ensure_ascii=False) + ";\n")
    shutil.copyfile(os.path.join(os.path.dirname(__file__), "board.html"), os.path.join(RUNS, "board.html"))
    # Optional mirror to the VPS board container (BOARD_REMOTE=deploy@<VPS_HOST>); never blocks the caller.
    remote = os.environ.get("BOARD_REMOTE")
    if remote:
        import subprocess
        try:
            subprocess.run(["scp", "-q", "-o", "BatchMode=yes", "-o", "ConnectTimeout=5",
                            os.path.join(RUNS, "board-data.js"), os.path.join(RUNS, "board.html"),
                            remote + ":/opt/stacks/procedworld/board/"], timeout=20, check=False)
        except (OSError, subprocess.SubprocessError):
            pass


def main(argv):
    board = load()
    if argv[:1] == ["--clear-done"]:
        board["cards"] = {k: c for k, c in board["cards"].items() if c["status"] != "done"}
        save(board)
        return 0
    if len(argv) < 2 or argv[1] not in STATUSES:
        print(__doc__)
        return 2
    card_id, status, rest = argv[0], argv[1], argv[2:]
    opts = dict(zip(rest[0::2], rest[1::2]))
    now = datetime.datetime.now().strftime("%H:%M")
    card = board["cards"].setdefault(card_id, {"id": card_id, "title": card_id, "created": now})
    card["status"] = status
    for flag, key in (("--title", "title"), ("--agent", "agent"), ("--branch", "branch"), ("--note", "note")):
        if flag in opts:
            card[key] = opts[flag]
    if status == "doing" and "started" not in card:
        card["started"] = now
    if status in ("done", "failed"):
        card["ended"] = now
    card["changed"] = now
    save(board)
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))

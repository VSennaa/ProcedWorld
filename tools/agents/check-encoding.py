"""Fail if Markdown docs have encoding damage: U+FFFD, CRLF, or accents replaced by '?'.

Usage: python tools/agents/check-encoding.py [files...]   (default: all docs/**/*.md and root .md)
The '?'-inside-a-word check catches text rewritten through an ANSI code page (e.g. PowerShell 5.1
without -Encoding utf8), which U+FFFD checks miss.
"""
import glob, re, sys

MARKER = "<!-- encoding-check: quotes damaged text -->"  # opt-out for docs quoting damage
WORD_Q = re.compile(r"[A-Za-zÀ-ú]\?[A-Za-zÀ-ú]")
files = sys.argv[1:] or sorted(glob.glob("docs/**/*.md", recursive=True) + glob.glob("*.md"))
bad = 0
for f in files:
    raw = open(f, "rb").read()
    try:
        text = raw.decode("utf-8")
    except UnicodeDecodeError as e:
        print(f"{f}: not UTF-8 ({e})"); bad += 1; continue
    problems = []
    if "�" in text: problems.append(f"U+FFFD x{text.count(chr(0xfffd))}")
    if b"\r\n" in raw: problems.append("CRLF")
    hits = WORD_Q.findall(text)
    if hits and MARKER not in text: problems.append(f"'?' inside words x{len(hits)} e.g. {hits[:3]}")
    if problems:
        print(f"{f}: " + "; ".join(problems)); bad += 1
print("encoding ok" if not bad else f"{bad} file(s) with encoding damage")
sys.exit(1 if bad else 0)

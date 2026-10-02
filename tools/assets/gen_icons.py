"""Generate original 48px ProcedWorld UI icons with Python's standard library only."""

import argparse
import html
import json
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
ASSETS = ROOT / "assets"
SIZE = 48
BEGIN = "<!-- icons-preview:start -->"
END = "<!-- icons-preview:end -->"

# id, Portuguese label, palette role. Shapes supplement color for color-blind readability.
ICONS = (
    ("comida", "Comida", "food"), ("producao", "Produção", "production"),
    ("riqueza", "Riqueza", "wealth"), ("conhecimento", "Conhecimento", "knowledge"),
    ("cultura", "Cultura", "culture"), ("metal", "Metal", "metal"),
    ("luxo", "Luxo", "luxury"), ("coesao", "Coesão", "cohesion"),
    ("legitimidade", "Legitimidade", "legitimacy"), ("estabilidade", "Estabilidade", "stability"),
    ("pressao-crise", "Pressão de crise", "crisis"), ("ameaca-guerra", "Ameaça de guerra", "war"),
    ("exposicao-ambiental", "Exposição ambiental", "environment"),
    ("populacao", "População", "population"), ("turno", "Turno", "time"),
    ("era", "Era", "time"), ("mandato", "Mandato", "legitimacy"),
    ("governador", "Governador", "legitimacy"), ("entropia", "Entropia", "crisis"),
    ("ledger-confianca", "Ledger: confiança", "cohesion"),
    ("ledger-ressentimento", "Ledger: ressentimento", "crisis"),
    ("ledger-divida", "Ledger: dívida", "wealth"), ("tratado", "Tratado", "cohesion"),
    ("guerra", "Guerra", "war"), ("pronto", "Pronto", "stability"),
    ("custo-ia", "Custo de IA", "ai"),
)


def tag(name, **attrs):
    return "<" + name + " " + " ".join(f'{key.replace("_", "-")}="{html.escape(str(value), quote=True)}"' for key, value in attrs.items()) + "/>"


def line(data, color):
    return tag("path", d=data, fill="none", stroke=color, stroke_width="3", stroke_linecap="round", stroke_linejoin="round")


def fill(data, color):
    return tag("path", d=data, fill=color, stroke=color, stroke_width="3", stroke_linecap="round", stroke_linejoin="round")


def body(icon, color):
    """Flat, rounded 3px symbols designed to retain their silhouette at 24px."""
    ink = color
    shapes = {
        "comida": line("M24 39V25 M24 25C14 23 12 14 14 9C22 10 25 16 24 25 M24 25C32 23 36 17 35 10C27 11 23 17 24 25", ink),
        "producao": line("M12 36H36 M16 36V26L24 18L32 26V36 M24 18V10 M20 10H28", ink),
        "riqueza": tag("circle", cx="24", cy="24", r="14", fill="none", stroke=ink, stroke_width="3") + line("M28 17C27 15 24 15 22 16C18 18 20 21 24 22C29 23 29 28 25 31C22 33 18 31 17 29 M24 13V35", ink),
        "conhecimento": line("M14 12C18 10 22 11 24 14V36C21 33 17 33 12 35V14C13 13 13 13 14 12 M34 12C30 10 26 11 24 14 M24 36C27 33 31 33 36 35V14C35 13 35 13 34 12", ink),
        "cultura": line("M14 36V16 M14 19C20 14 28 14 34 19V33C28 28 20 28 14 33 M22 13V35", ink),
        "metal": fill("M24 9L37 17V31L24 39L11 31V17Z", ink) + line("M24 9V39 M11 17L24 25L37 17", "#F5F1E6"),
        "luxo": line("M11 17H37L32 35H16Z M16 17L20 10L24 17L28 10L32 17 M18 24H30", ink),
        "coesao": tag("circle", cx="17", cy="24", r="7", fill="none", stroke=ink, stroke_width="3") + tag("circle", cx="31", cy="24", r="7", fill="none", stroke=ink, stroke_width="3") + line("M22 24H26", ink),
        "legitimidade": line("M24 9L37 15V24C37 32 31 37 24 40C17 37 11 32 11 24V15Z M18 24L22 28L30 19", ink),
        "estabilidade": line("M12 24L19 31L36 14 M12 35H36", ink),
        "pressao-crise": line("M24 9L39 36H9Z M24 18V27 M24 32V33", ink),
        "ameaca-guerra": line("M24 9V29 M17 15L24 9L31 15 M15 39L20 29H28L33 39 M13 35H35", ink),
        "exposicao-ambiental": line("M24 39C14 33 12 22 16 13C24 14 31 19 32 29C31 35 27 38 24 39 M15 37C22 30 26 24 32 18", ink),
        "populacao": tag("circle", cx="24", cy="16", r="6", fill="none", stroke=ink, stroke_width="3") + line("M12 38C12 30 17 26 24 26C31 26 36 30 36 38", ink),
        "turno": tag("circle", cx="24", cy="24", r="15", fill="none", stroke=ink, stroke_width="3") + line("M24 15V24L30 28", ink),
        "era": line("M12 35V13H36 M17 13V35 M12 24H32 M28 20L36 24L28 28", ink),
        "mandato": line("M16 10H32V38H16Z M20 17H28 M20 24H28 M20 31H25", ink),
        "governador": tag("circle", cx="24", cy="16", r="6", fill="none", stroke=ink, stroke_width="3") + line("M13 38C13 31 18 27 24 27C30 27 35 31 35 38 M13 12L24 8L35 12", ink),
        "entropia": line("M14 11H34 M15 37H33 M18 11C18 18 30 19 30 24C30 29 18 30 18 37 M30 11C30 18 18 19 18 24C18 29 30 30 30 37", ink),
        "ledger-confianca": line("M12 18C16 12 21 13 24 18C27 13 32 12 36 18C38 24 30 31 24 36C18 31 10 24 12 18Z", ink),
        "ledger-ressentimento": line("M14 35L34 13 M14 13L34 35 M10 24H38", ink),
        "ledger-divida": line("M13 13H35V35H13Z M18 20H30 M18 27H26 M31 31L36 36 M36 31L31 36", ink),
        "tratado": line("M11 17H23V31H11Z M25 17H37V31H25Z M23 24H25 M16 22V26 M32 22V26", ink),
        "guerra": line("M15 12L34 31 M30 11L36 17 M11 30L17 36 M13 36L36 13 M11 17L17 11 M30 36L36 30", ink),
        "pronto": tag("circle", cx="24", cy="24", r="15", fill="none", stroke=ink, stroke_width="3") + line("M16 24L21 29L33 18", ink),
        "custo-ia": line("M15 15H33V33H15Z M20 15V11 M28 15V11 M20 33V37 M28 33V37 M15 20H11 M15 28H11 M33 20H37 M33 28H37 M20 24H28", ink),
    }
    return shapes[icon]


def svg(icon, label, color):
    return (f'<svg xmlns="http://www.w3.org/2000/svg" width="48" height="48" viewBox="0 0 48 48" role="img">'
            f'<title>{html.escape(label)}</title>{body(icon, color)}</svg>\n')


def write(path, content):
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(content, encoding="utf-8", newline="\n")


def preview(palette):
    panels = []
    for theme in ("light", "dark"):
        galleries = []
        for mode, heading in (("standard", "Paleta padrão"), ("colorblind", "Alternativa para daltonismo"), ("monochrome", "Status monocromático")):
            cards = "".join(
                f'<figure><img src="icons/{mode}/{icon}.svg" alt="{label}" width="48" height="48"><figcaption>{label}</figcaption></figure>'
                for icon, label, _ in ICONS)
            galleries.append(f'<h3>{heading}</h3><div class="icon-gallery">{cards}</div>')
        panels.append(f'<section class="icon-panel {theme}"><h2>Fundo {"claro" if theme == "light" else "escuro"}</h2>{"".join(galleries)}</section>')
    colors = palette["preview"]
    return BEGIN + f'''\n<style>
#icons-preview {{font:16px/1.5 system-ui,sans-serif; background:{colors['light']}; color:{colors['light_ink']}; padding:24px;}}
#icons-preview * {{box-sizing:border-box;}} .icon-panels {{display:grid; grid-template-columns:repeat(2,minmax(0,1fr)); gap:20px;}}
.icon-panel {{padding:18px; border:1px solid #83949b; border-radius:16px; min-width:0;}} .icon-panel.light {{background:{colors['light']}; color:{colors['light_ink']};}} .icon-panel.dark {{background:{colors['dark']}; color:{colors['dark_ink']};}}
.icon-gallery {{display:grid; grid-template-columns:repeat(auto-fit,minmax(92px,1fr)); gap:8px;}} .icon-gallery figure {{margin:0; min-height:88px; padding:8px 4px; text-align:center; border-radius:8px;}} .icon-panel.light figure {{background:#fffdf6;}} .icon-panel.dark figure {{background:#20323d;}} .icon-gallery img {{display:block; margin:auto;}} .icon-gallery figcaption {{font-size:12px; line-height:1.2; margin-top:6px;}}
@media(max-width:760px) {{#icons-preview {{padding:12px;}} .icon-panels {{grid-template-columns:1fr;}}}}
</style>
<section id="icons-preview"><p>PROCEDWORLD / ESTUDO DE UI / 02</p><h1>Recursos, estado e decisão</h1><p>Ícones planos de 48 × 48, desenhados para permanecer legíveis a 24 × 24. Forma e cor carregam significado; a versão monocromática serve estados e superfícies compactas.</p><div class="icon-panels">{"".join(panels)}</div></section>
''' + END


def replace_section(text, section):
    if BEGIN in text and END in text:
        return text[:text.index(BEGIN)] + section + text[text.index(END) + len(END):]
    return text.replace("</body>", section + "\n</body>")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--seed", type=int, default=20261001, help="Recorded in the manifest; icon geometry is fixed.")
    args = parser.parse_args()
    palette = json.loads((ASSETS / "palette.json").read_text(encoding="utf-8"))
    for mode in ("standard", "colorblind"):
        for icon, label, role in ICONS:
            write(ASSETS / "icons" / mode / f"{icon}.svg", svg(icon, label, palette["icons"][mode][role]))
    for icon, label, _ in ICONS:
        write(ASSETS / "icons" / "monochrome" / f"{icon}.svg", svg(icon, label, palette["icons"]["monochrome"]))
    manifest = {"version": 1, "seed": args.seed, "size": SIZE, "viewBox": "0 0 48 48", "stroke_width": 3,
                "modes": ["standard", "colorblind", "monochrome"],
                "icons": [{"id": icon, "label": label, "palette_role": role} for icon, label, role in ICONS]}
    write(ASSETS / "icons" / "manifest.json", json.dumps(manifest, ensure_ascii=False, indent=2) + "\n")
    target = ASSETS / "preview.html"
    write(target, replace_section(target.read_text(encoding="utf-8"), preview(palette)))


if __name__ == "__main__":
    main()

"""Generate deterministic unit, building, city and tile-improvement SVG assets."""

import argparse
import hashlib
import html
import json
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
ASSETS = ROOT / "assets"
ENTITIES = ASSETS / "entities"
BEGIN = "<!-- entities-preview:start -->"
END = "<!-- entities-preview:end -->"
VARIANTS = ("standard", "colorblind", "monochrome")
IMPROVEMENTS = (("fazenda", "Fazenda", "food"), ("mina", "Mina", "metal"),
                ("pastagem", "Pastagem", "food"), ("serraria", "Serraria", "production"),
                ("porto", "Porto", "knowledge"), ("estrada", "Estrada", "time"))
CITY_SIZES = (("pequena", "Pequena", 12), ("media", "Média", 16),
              ("grande", "Grande", 21), ("metropole", "Metrópole", 26))
CITY_STATES = (("normal", "Normal"), ("protesto", "Protesto"),
               ("revolta", "Revolta"), ("cerco", "Cerco"))


def tag(name, **attrs):
    return "<" + name + " " + " ".join(
        f'{key.replace("_", "-")}="{html.escape(str(value), quote=True)}"'
        for key, value in attrs.items()) + "/>"


def path(d, color, width=3, fill="none"):
    return tag("path", d=d, fill=fill, stroke=color, stroke_width=width,
               stroke_linecap="round", stroke_linejoin="round")


def glyph(identity, category, color, accent):
    """Compact pictograms use catalog roles and stable hashes for shape variants."""
    key = identity.rsplit(".", 1)[-1]
    if category == "unit":
        if any(word in key for word in ("scout", "pathfinder")):
            return path("M24 7L38 38L24 32L10 38Z M24 8V32 M16 23H32", color)
        if any(word in key for word in ("militia", "guard", "raider", "siege")):
            return path("M24 7L37 13V24C37 31 31 37 24 41C17 37 11 31 11 24V13Z M18 26L30 18 M27 16L32 21", color)
        if "settler" in key:
            return path("M24 8L37 19H31V39H17V19H11Z M22 39V27H27V39", color)
        if "worker" in key:
            return path("M13 35L31 17 M27 13L35 21 M11 37H37 M18 29L24 35", color)
        return path("M8 23H40 M12 18L24 10L36 18V36H12Z M20 36V27H28V36", color)
    if category == "building":
        shapes = {
            "food_storage": "M9 18L24 9L39 18V38H9Z M17 38V25H31V38 M13 21H35",
            "city_defense": "M9 38V15L16 20V13L24 19V11L32 18V13L39 18V38Z M19 38V29H29V38",
            "production": "M10 38V23L18 28V20L26 25V13L38 9V38Z M29 18H34",
            "governance": "M9 19L24 9L39 19 M12 21H36 M15 21V35 M22 21V35 M29 21V35 M10 38H38",
            "health": "M12 12H36V37H12Z M24 17V31 M17 24H31",
            "trade": "M10 14H38V36H10Z M15 20H33 M15 26H29 M15 32H24",
            "knowledge": "M12 12C17 10 21 12 24 15V37C21 33 17 33 12 35Z M36 12C31 10 27 12 24 15V37C27 33 31 33 36 35Z",
            "water_management": "M24 8C18 17 13 22 13 29A11 11 0 0 0 35 29C35 22 30 17 24 8Z M19 30C20 34 23 35 27 34",
            "transport": "M9 31H39 M13 31V20L24 12L35 20V31 M18 31V24H30V31 M8 37H40",
        }
        catalog = json.loads((ROOT / "data/catalogs/buildings.json").read_text(encoding="utf-8"))
        item = next((x for x in catalog["buildings"] if x["id"] == identity), {})
        return path(shapes.get(item.get("function"), shapes["production"]), color)
    if category == "improvement":
        shapes = {
            "fazenda": "M9 35H39 M13 35V25L24 18L35 25V35 M18 28H30 M24 18V10 M19 13L24 18L29 13",
            "mina": "M10 37L19 22L25 29L32 15L39 37Z M13 34H36 M30 10L35 15L30 20",
            "pastagem": "M12 36C10 26 17 19 24 16C31 19 38 26 36 36 M24 36V20 M17 27L24 32L31 27",
            "serraria": "M10 35H38 M13 29L19 12L25 29 M23 29L29 12L35 29 M16 35V30 M32 35V30",
            "porto": "M9 28H39L34 36H15Z M16 24V12H30L35 19 M20 16H28 M12 40C17 37 21 43 26 40C31 37 35 43 40 40",
            "estrada": "M9 37L20 11H28L39 37 M18 29H30 M21 21H27",
        }
        return path(shapes[key], color)
    raise ValueError(category)


def svg(label, body, seed, identity, variant):
    return (f'<svg xmlns="http://www.w3.org/2000/svg" width="48" height="48" viewBox="0 0 48 48" role="img">'
            f'<title>{html.escape(label)}</title>{body}</svg>\n')


def city_svg(size, label, color, state, seed, variant, ink):
    r = size
    parts = [tag("circle", cx=24, cy=24, r=r, fill=color, stroke=ink, stroke_width=3),
             path("M13 27L24 17L35 27 M16 25V35H32V25 M22 35V28H27V35", ink, 2.7)]
    if state == "protesto":
        parts.append(path("M8 9L14 9L15 17 M14 9L18 13", "#C66A34" if variant != "monochrome" else ink, 3))
    elif state == "revolta":
        parts.append(path("M24 5L28 12L24 19L20 12Z", "#B84343" if variant != "monochrome" else ink, 2.5, "#B84343" if variant != "monochrome" else "none"))
    elif state == "cerco":
        parts.extend((tag("circle", cx=24, cy=24, r=r + 4, fill="none", stroke="#B84343" if variant != "monochrome" else ink, stroke_width=2.5, stroke_dasharray="4 3"),
                      path("M35 8L40 13L35 18", "#B84343" if variant != "monochrome" else ink, 3)))
    return svg(f"Cidade {label.lower()} — {state}", "".join(parts), seed, f"city-{size}-{state}", variant)


def slug(value):
    value = value.lower().replace("ç", "c").replace("ã", "a").replace("é", "e")
    return re.sub(r"[^a-z0-9]+", "-", value).strip("-")


def preview(items):
    cards = []
    for title, folder, records in items:
        cards.append(f'<h3>{html.escape(title)}</h3><div class="entity-gallery">' + "".join(
            f'<figure><img src="entities/standard/{folder}/{slug(identity)}.svg" alt="{html.escape(label)}" width="48" height="48"><figcaption>{html.escape(label)}</figcaption></figure>'
            for identity, label in records) + "</div>")
    city_cards = "".join(f'<figure><img src="entities/standard/cidades/cidade-{slug(size)}-{state}.svg" alt="Cidade {label}, {state}" width="48" height="48"><figcaption>{label} · {state}</figcaption></figure>' for size, label, _ in CITY_SIZES for state, _ in CITY_STATES)
    return BEGIN + '''
<style>
#entities-preview {font:16px/1.5 system-ui,sans-serif;color:#202E38;background:#F5F1E6;padding:24px} #entities-preview *{box-sizing:border-box}
#entities-preview h1{font-size:clamp(26px,4vw,44px);margin:0}.entity-gallery{display:grid;grid-template-columns:repeat(auto-fit,minmax(100px,1fr));gap:8px;margin:12px 0 24px}
.entity-gallery figure{margin:0;text-align:center;padding:10px 4px;background:#fffdf6;border-radius:8px}.entity-gallery img{display:block;margin:auto}.entity-gallery figcaption{font-size:12px;line-height:1.2;margin-top:6px}
</style><section id="entities-preview"><p>PROCEDWORLD / ESTUDO DE ENTIDADES / 03</p><h1>Unidades, edifícios e cidades</h1><p>Ícones vetoriais planos de 48 × 48, legíveis a 24 × 24. Cor de civilização é parâmetro nos marcadores; estados também têm sinais de forma.</p>
''' + "".join(cards) + f'<h3>Marcadores por população e estado</h3><div class="entity-gallery">{city_cards}</div></section>\n' + END


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--seed", type=int, default=20261002)
    parser.add_argument("--civilization-color", default="#56B4E9")
    args = parser.parse_args()
    if not re.fullmatch(r"#[0-9A-Fa-f]{6}", args.civilization_color):
        parser.error("--civilization-color deve usar #RRGGBB")
    palette = json.loads((ASSETS / "palette.json").read_text(encoding="utf-8"))["icons"]
    catalogs = []
    for filename, key, category in (("units.json", "units", "unit"), ("buildings.json", "buildings", "building")):
        data = json.loads((ROOT / "data/catalogs" / filename).read_text(encoding="utf-8"))
        records = [(category, x["id"], x["name"], x.get("role", x.get("function", "production"))) for x in data[key]]
        catalogs.append((category, records))
    for category, records in catalogs:
        for mode in VARIANTS:
            colors = palette[mode] if mode != "monochrome" else {"ink": palette["monochrome"]}
            for _, identity, label, role in records:
                digest = hashlib.sha256(f"entities-v1|{args.seed}|{identity}".encode()).hexdigest()
                color = colors.get({"exploration": "knowledge", "defense": "legitimacy", "attack": "war", "settler": "population", "worker": "production", "trade": "wealth"}.get(role, role), colors["ink"])
                body = glyph(identity, category, color, colors["ink"])
                path = ENTITIES / mode / ("unidades" if category == "unit" else "edificios") / f"{slug(identity)}.svg"
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_text(svg(label, body, args.seed, identity, mode), encoding="utf-8", newline="\n")
    improvements = [("improvement", key, label, role) for key, label, role in IMPROVEMENTS]
    for mode in VARIANTS:
        colors = palette[mode] if mode != "monochrome" else {"ink": palette["monochrome"]}
        for _, identity, label, role in improvements:
            color = colors.get(role, colors["ink"])
            out = ENTITIES / mode / "melhorias" / f"{identity}.svg"
            out.parent.mkdir(parents=True, exist_ok=True)
            out.write_text(svg(label, glyph(identity, "improvement", color, colors["ink"]), args.seed, identity, mode), encoding="utf-8", newline="\n")
        for size, label, radius in CITY_SIZES:
            for state, _ in CITY_STATES:
                name = f"cidade-{size}-{state}"
                out = ENTITIES / mode / "cidades" / f"{name}.svg"
                out.parent.mkdir(parents=True, exist_ok=True)
                out.write_text(city_svg(radius, label, args.civilization_color if mode != "monochrome" else palette["monochrome"], state, args.seed, mode, colors["ink"]), encoding="utf-8", newline="\n")
    manifest = {"generator": "entities-v1", "seed": args.seed, "dimensions": [48, 48], "variants": list(VARIANTS), "civilization_color": args.civilization_color, "unit_catalog": "core.units", "building_catalog": "core.buildings", "improvements": [x[0] for x in IMPROVEMENTS], "city_population_sizes": [{"id": slug(k), "label": v, "radius": r} for k, v, r in CITY_SIZES], "city_states": [x[0] for x in CITY_STATES]}
    (ENTITIES / "manifest.json").write_text(json.dumps(manifest, ensure_ascii=False, indent=2) + "\n", encoding="utf-8", newline="\n")
    preview_path = ASSETS / "preview.html"
    page = preview_path.read_text(encoding="utf-8")
    preview_items = [("Unidades", "unidades", [(x[1], x[2]) for x in catalogs[0][1]]),
                     ("Edifícios", "edificios", [(x[1], x[2]) for x in catalogs[1][1]]),
                     ("Melhorias de tile", "melhorias", [(x[0], x[1]) for x in IMPROVEMENTS])]
    section = preview(preview_items)
    if BEGIN in page and END in page:
        page = page[:page.index(BEGIN)] + section + page[page.index(END) + len(END):]
    else:
        marker = "<!-- icons-preview:end -->"
        page = page.replace(marker, marker + "\n" + section, 1)
    preview_path.write_text(page, encoding="utf-8", newline="\n")


if __name__ == "__main__":
    main()

"""Generate original CC0 hex illustrations using only the Python standard library."""

import argparse
import hashlib
import html
import json
import math
from pathlib import Path
import re


ROOT = Path(__file__).resolve().parents[2]
ASSETS = ROOT / "assets"
RADIUS = 64
WIDTH = math.sqrt(3) * RADIUS
HEIGHT = 128
VERTICES = [(WIDTH / 2, 0), (WIDTH, 32), (WIDTH, 96),
            (WIDTH / 2, 128), (0, 96), (0, 32)]
BIOMES = dict(zip(
    ("oceano", "costa", "planicie", "floresta", "selva", "savana",
     "deserto", "estepe", "tundra", "pantano", "montanha", "geleira"),
    ("Oceano", "Costa", "Planície", "Floresta", "Selva", "Savana",
     "Deserto", "Estepe", "Tundra", "Pântano", "Montanha", "Geleira")))
EDGES = ("ne", "e", "se", "sw", "w", "nw")
BEGIN = "<!-- tiles-preview:start -->"
END = "<!-- tiles-preview:end -->"


def number(value):
    return f"{value:.9f}".rstrip("0").rstrip(".")


def points(vertices):
    return " ".join(f"{number(x)},{number(y)}" for x, y in vertices)


HEX = points(VERTICES)


def sample(seed, *keys):
    """Version 1: stable, independent SHA-256 samples; no process hash or PRNG state."""
    data = json.dumps(["tiles-v1", seed, *keys], ensure_ascii=True, separators=(",", ":"))
    return int.from_bytes(hashlib.sha256(data.encode("ascii")).digest()[:8], "big")


def choose_variant(seed, biome, column, row):
    return 1 + sample(seed, biome, column, row) % 3


def svg(body, title, width=WIDTH, height=HEIGHT):
    return (f'<svg xmlns="http://www.w3.org/2000/svg" width="{number(width)}" '
            f'height="{number(height)}" viewBox="0 0 {number(width)} {number(height)}" '
            f'role="img"><title>{html.escape(title)}</title>{body}</svg>\n')


def path(data, color, width=2, **attrs):
    extra = " ".join(f'{key.replace("_", "-")}="{value}"' for key, value in attrs.items())
    return (f'<path d="{data}" fill="none" stroke="{color}" stroke-width="{width}" '
            f'stroke-linecap="round" stroke-linejoin="round" {extra}/>')


def polygon(vertices, color):
    return f'<polygon points="{points(vertices)}" fill="{color}"/>'


def ellipse(x, y, rx, ry, color):
    return f'<ellipse cx="{x}" cy="{y}" rx="{rx}" ry="{ry}" fill="{color}"/>'


def motif(biome, ink, light):
    if biome in ("oceano", "costa"):
        return path("M-13 0 Q-7 -5 0 0 T13 0", ink, 2.5) + path("M-8 7 Q-3 3 3 7 T13 7", ink, 1.5)
    if biome == "planicie":
        return path("M-9 7 Q-3 0 10 4 M-2 3 L-4 -3 M3 3 L6 -3", ink, 2.5)
    if biome == "floresta":
        return (path("M0 5 V14", ink, 3) + polygon([(-12, 7), (0, -17), (12, 7)], ink)
                + polygon([(-8, -1), (0, -17), (0, 3)], light))
    if biome == "selva":
        return (path("M0 0 V14 M0 6 Q12 14 10 -2", ink, 3)
                + ellipse(-7, -3, 9, 10, ink) + ellipse(5, -7, 11, 12, ink)
                + ellipse(1, -11, 6, 5, light))
    if biome == "savana":
        return (path("M2 13 L0 -4 M0 5 L-8 -2 M0 2 L9 -3", ink, 3)
                + ellipse(0, -5, 17, 5, ink) + path("M-10 -7 Q0 -11 11 -7", light, 2))
    if biome == "deserto":
        return (path("M-18 9 Q-4 -13 17 5 Q5 1 -2 9", ink, 2.5)
                + path("M-9 3 Q0 -4 9 1", light, 3))
    if biome == "estepe":
        return path("M-11 8 L-14 2 M-8 8 V-1 M-5 8 L-2 2 M5 4 L3 -3 M8 4 L10 -4", ink, 2.5)
    if biome == "tundra":
        return (ellipse(0, 6, 14, 4, light) + polygon([(-9, 7), (-5, -2), (3, -4), (10, 7)], ink)
                + path("M-15 -5 H-9 M-12 -8 V-2", ink, 2))
    if biome == "pantano":
        return (ellipse(0, 9, 17, 5, ink) + path("M-12 10 H10", light, 2)
                + path("M-6 7 V-12 M1 7 V-7 M8 7 V-13", ink, 2)
                + path("M-6 -13 V-7 M8 -14 V-8", light, 3))
    if biome == "montanha":
        return (polygon([(-20, 15), (0, -21), (20, 15)], ink)
                + polygon([(0, -21), (20, 15), (4, 9)], light)
                + polygon([(-7, -8), (0, -21), (7, -8), (0, -11)], "#FFFFFF"))
    return (polygon([(-16, 10), (-10, -12), (2, -18), (15, -5), (17, 12)], light)
            + path("M-10 -12 L-4 2 L-16 10 M-4 2 L2 -18 M-4 2 L17 12", ink, 2.5))


def tile_body(biome, variant, seed, colors):
    base, ink, light = colors
    body = polygon(VERTICES, base)
    if biome == "costa":
        body += f'<path d="M0 32 L{number(WIDTH / 2)} 0 L{number(WIDTH)} 32 V49 Q65 28 0 76Z" fill="{light}"/>'
    positions = [(36, 39), (75, 49), (34, 77), (71, 91)]
    if biome == "montanha":
        positions = [(39, 55), (73, 80)]
    for index, (x, y) in enumerate(positions):
        x += sample(seed, biome, variant, index, "x") % 9 - 4
        y += sample(seed, biome, variant, index, "y") % 7 - 3
        scale = (88 + sample(seed, biome, variant, index, "s") % 20) / 100
        body += f'<g transform="translate({x} {y}) scale({scale})">{motif(biome, ink, light)}</g>'
    return body


def edge_path(edge):
    a, b = VERTICES[edge], VERTICES[(edge + 1) % 6]
    return f"M{number(a[0])} {number(a[1])} L{number(b[0])} {number(b[1])}"


def overlay_bodies(palette, civilization):
    colors = palette["overlays"]
    clip = f'<defs><clipPath id="hex"><polygon points="{HEX}"/></clipPath></defs>'
    result = {}
    for edge, name in enumerate(EDGES):
        river = path(edge_path(edge), colors["river"], 9) + path(edge_path(edge), colors["river_highlight"], 3)
        result[f"rio-{name}"] = clip + f'<g clip-path="url(#hex)">{river}</g>'
        border = path(edge_path(edge), colors["border_halo"], 9)
        border += path(edge_path(edge), civilization, 5, stroke_dasharray="9 5")
        result[f"fronteira-{name}"] = clip + f'<g clip-path="url(#hex)">{border}</g>'
    result["fronteira"] = clip + '<g clip-path="url(#hex)">' + path(
        "M" + " L".join(f"{number(x)} {number(y)}" for x, y in VERTICES) + "Z",
        colors["border_halo"], 9) + path(
        "M" + " L".join(f"{number(x)} {number(y)}" for x, y in VERTICES) + "Z",
        civilization, 5, stroke_dasharray="9 5") + '</g>'
    clock = (ellipse(55, 64, 10, 10, colors["fog_mark"])
             + path("M55 57 V64 L61 67", colors["fog"], 2.5))
    result["nevoa-lembrado"] = f'<polygon points="{HEX}" fill="{colors["fog"]}" opacity="0.58"/>' + clock
    result["nevoa-desconhecido"] = polygon(VERTICES, colors["fog"]) + path(
        "M44 57 Q44 46 55 46 Q67 46 67 56 Q67 62 55 67 V72 M55 81 V83", colors["fog_mark"], 4)
    return result


def write(pathname, content):
    pathname.parent.mkdir(parents=True, exist_ok=True)
    with pathname.open("w", encoding="utf-8", newline="\n") as target:
        target.write(content)


def minimap(seed, mode, bodies, palette):
    # Demonstration layout only: not the world's biome generation algorithm.
    rows = [
        "0011aabb00", "0018bbaa10", "011882aa30", "0162244330",
        "0162255330", "0177955330", "0017996610", "0001166100"]
    names = list(BIOMES)
    bases, decorations = [], []
    for row, cells in enumerate(rows):
        for column, code in enumerate(cells):
            biome = names[int(code, 12)]
            variant = choose_variant(seed, biome, column, row)
            x, y = WIDTH * (column + 0.5 * (row % 2)), 96 * row
            transform = f'transform="translate({number(x)} {y})"'
            base = palette["tiles"][mode][biome][0]
            # A subpixel underpaint prevents rasterizer hairlines at fractional zoom.
            bases.append(f'<polygon {transform} points="{HEX}" fill="{base}" stroke="{base}" stroke-width="0.8"/>')
            decorations.append(f'<g {transform}>{bodies[mode, biome, variant]}</g>')
    return svg("".join(bases + decorations), "Mini-mapa ilustrativo 10 × 8, grid odd-r", WIDTH * 10.5, 800)


def preview(palette, overlays):
    panels = []
    for theme in ("light", "dark"):
        groups = []
        for mode, label in (("standard", "Padrão"), ("colorblind", "Alternativa para daltonismo")):
            cards = []
            for biome, name in BIOMES.items():
                images = "".join(f'<img src="tiles/{mode}/{biome}-{v}.svg" alt="{name}, variação {v}" width="83" height="96">' for v in range(1, 4))
                cards.append(f'<figure><div class="tile-variants">{images}</div><figcaption>{name} · 1 / 2 / 3</figcaption></figure>')
            groups.append(f'<h3>{label}</h3><div class="tile-gallery">{"".join(cards)}</div>'
                          f'<img class="tile-map" src="tiles/minimapa-{mode}.svg" alt="Mini-mapa 10 por 8, paleta {label}">')
        cards = []
        for name in overlays:
            cards.append(f'<figure><div class="tile-stack"><img src="tiles/standard/planicie-1.svg" alt="">'
                         f'<img src="tiles/overlays/{name}.svg" alt="{name}"></div><figcaption>{name}</figcaption></figure>')
        groups.append('<h3>Overlays sobre planície</h3><div class="tile-gallery overlays">' + "".join(cards) + '</div>')
        panels.append(f'<section class="tile-panel {theme}"><h2>Fundo {"claro" if theme == "light" else "escuro"}</h2>{"".join(groups)}</section>')
    c = palette["preview"]
    return BEGIN + f'''
<style>
#tiles-preview {{font:16px/1.5 system-ui,sans-serif; color:{c['light_ink']}; background:{c['light']}; padding:24px;}}
#tiles-preview * {{box-sizing:border-box;}}
#tiles-preview h1 {{font-size:clamp(26px,4vw,44px); margin:0;}}
.tile-panels {{display:grid; grid-template-columns:repeat(2,minmax(0,1fr)); gap:20px;}}
.tile-panel {{padding:18px; border-radius:16px; border:1px solid #83949b; min-width:0;}}
.tile-panel.light {{background:{c['light']}; color:{c['light_ink']};}}
.tile-panel.dark {{background:{c['dark']}; color:{c['dark_ink']};}}
.tile-gallery {{display:grid; grid-template-columns:repeat(auto-fit,minmax(255px,1fr)); gap:12px;}}
.tile-gallery figure {{margin:0; padding:12px 0; text-align:center;}}
.tile-variants {{display:flex; justify-content:center;}}
.tile-gallery figcaption {{font-size:14px; margin-top:8px;}}
.tile-map {{display:block; width:100%; height:auto; margin:24px 0;}}
.tile-gallery.overlays {{grid-template-columns:repeat(auto-fit,minmax(130px,1fr));}}
.tile-stack {{position:relative; width:83px; height:96px; margin:auto;}}
.tile-stack img {{position:absolute; inset:0; width:100%; height:100%;}}
@media(max-width:760px) {{.tile-panels {{grid-template-columns:1fr;}} #tiles-preview {{padding:12px;}}}}
</style>
<section id="tiles-preview">
<p>PROCEDWORLD / ESTUDO DE TERRENO / 01</p><h1>Um mundo em doze biomas</h1>
<p>Hexágonos ilustrados · três variações · formas reconhecíveis além da cor · obra própria CC0.</p>
<p>Mapas ilustrativos 10 × 8. Névoa lembrada: relógio e transparência; desconhecida: cobertura opaca.
Fronteira tracejada; rio contínuo. Arestas NE, E, SE, SW, W, NW no sentido horário.</p>
<div class="tile-panels">{''.join(panels)}</div></section>
''' + END


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--seed", type=int, default=20261001)
    parser.add_argument("--civilization-color", default=None, help="Border color in #RRGGBB format")
    args = parser.parse_args()
    palette = json.loads((ASSETS / "palette.json").read_text(encoding="utf-8"))
    civilization = args.civilization_color or palette["overlays"]["civilizations"][0]
    if not re.fullmatch(r"#[0-9A-Fa-f]{6}", civilization):
        parser.error("--civilization-color must be #RRGGBB")
    bodies = {}
    for mode in ("standard", "colorblind"):
        for biome, label in BIOMES.items():
            for variant in range(1, 4):
                body = tile_body(biome, variant, args.seed, palette["tiles"][mode][biome])
                bodies[mode, biome, variant] = body
                write(ASSETS / "tiles" / mode / f"{biome}-{variant}.svg", svg(body, f"{label} · {variant}"))
        write(ASSETS / "tiles" / f"minimapa-{mode}.svg", minimap(args.seed, mode, bodies, palette))
    overlays = overlay_bodies(palette, civilization)
    for name, body in overlays.items():
        write(ASSETS / "tiles" / "overlays" / f"{name}.svg", svg(body, name))
    manifest = {"version": 1, "seed": args.seed, "radius": RADIUS, "width": WIDTH,
                "height": HEIGHT, "grid": "odd-r", "edges_clockwise": EDGES,
                "civilization_color": civilization, "biomes": BIOMES,
                "variants": [1, 2, 3], "modes": ["standard", "colorblind"]}
    write(ASSETS / "tiles" / "manifest.json", json.dumps(manifest, ensure_ascii=False, indent=2) + "\n")
    target = ASSETS / "preview.html"
    section = preview(palette, overlays)
    existing = target.read_text(encoding="utf-8") if target.exists() else ""
    if BEGIN in existing and END in existing:
        start, end = existing.index(BEGIN), existing.index(END) + len(END)
        document = existing[:start] + section + existing[end:]
    elif "</body>" in existing:
        document = existing.replace("</body>", section + "\n</body>", 1)
    elif existing:
        document = existing + "\n" + section
    else:
        document = ('<!doctype html>\n<html lang="pt-BR"><head><meta charset="utf-8">'
                    '<meta name="viewport" content="width=device-width,initial-scale=1">'
                    '<title>ProcedWorld — atlas de biomas</title></head><body style="margin:0">\n'
                    + section + '\n</body></html>\n')
    write(target, document)
    print(f"Generated 72 biome SVGs, {len(overlays)} overlays and 2 maps; seed={args.seed}.")


if __name__ == "__main__":
    main()

extends RefCounted
## Adapter from the server's per-civilization view (`view_for` in pw-server/src/view.rs, as pushed in
## `state_snapshot` and in `turn_diff.view`) to the client's WorldView model.
## The client never computes rules: tiles, cities, units, ownership and orders are shown as sent.
## Optional fields (`order`, `skipped_turn`, `idle_units`, `map_height`) may be missing, so the adapter
## also works with servers that predate the unit-order contract.

const Hex := preload("res://scripts/hex.gd")
const WorldView := preload("res://scripts/world_view.gd")

## Catalog biome ids are English; map assets retain their Portuguese filenames.
const BIOME_ASSETS := {"plains": "planicie", "forest": "floresta", "jungle": "selva", "swamp": "pantano", "desert": "deserto", "steppe": "estepe", "coast": "costa", "ocean": "oceano"}
## assets/palette.json overlays.civilizations, repeated for more than four civilizations.
const CIV_COLORS: Array[String] = ["#E69F00", "#56B4E9", "#CC79A7", "#F0E442"]
const KNOWN_ORDERS: Array[String] = ["Idle", "Fortify", "Explore", "MoveTo", "Sentry", "Build"]
## Fallback role when no catalog has arrived yet (catalog ids are authoritative otherwise).
const ROLE_BY_TYPE := {
	"unit.scout": "exploration", "unit.pathfinder": "exploration", "unit.militia": "defense",
	"unit.guard": "defense", "unit.raider": "attack", "unit.siege_crew": "attack",
	"unit.settler": "settler", "unit.worker": "worker", "unit.caravan": "trade", "unit.coastal_trader": "trade",
}


static func civ_color(civ: int) -> Color:
	return Color(CIV_COLORS[posmod(civ, CIV_COLORS.size())])


## "event.valley_drought" -> "Valley drought" (display fallback; the server sends ids, not prose).
static func humanize(id: String) -> String:
	var text := id.get_slice(".", id.get_slice_count(".") - 1).replace("_", " ")
	return text.substr(0, 1).to_upper() + text.substr(1)


## Parses the serialized `UnitOrder`: {"type": "idle"|"fortify"|"explore"|"sentry"} or
## {"type": "move_to", "data": {"target": N}} (engine serde). The older externally tagged
## form ("Idle" | {"MoveTo": {...}}) is still accepted.
## Returns {kind, target}; kind is "" when the field is absent or unrecognized.
static func parse_order(raw: Variant, width: int) -> Dictionary:
	var none := Vector2i(-1, -1)
	if typeof(raw) == TYPE_DICTIONARY and typeof(raw.get("type")) == TYPE_STRING:
		var tag: String = raw["type"]
		if tag == "move_to":
			var data: Variant = raw.get("data", {})
			raw = {"MoveTo": data if typeof(data) == TYPE_DICTIONARY else {}}
		elif tag == "build":
			var data: Variant = raw.get("data", {})
			return {"kind": "Build", "target": none, "improvement": String(data.get("improvement", "")) if typeof(data) == TYPE_DICTIONARY else ""}
		else:
			raw = {"idle": "Idle", "fortify": "Fortify", "explore": "Explore", "sentry": "Sentry"}.get(tag, "")
	if typeof(raw) == TYPE_STRING and KNOWN_ORDERS.has(raw) and raw != "MoveTo":
		return {"kind": raw, "target": none}
	if typeof(raw) == TYPE_DICTIONARY and raw.has("MoveTo") and typeof(raw["MoveTo"]) == TYPE_DICTIONARY:
		var target: Variant = raw["MoveTo"].get("target")
		if (typeof(target) == TYPE_INT or typeof(target) == TYPE_FLOAT) and width > 0:
			return {"kind": "MoveTo", "target": Hex.cell_of_index(int(target), width)}
	return {"kind": "", "target": none}


## Builds a WorldView from a `view_for` payload. `catalog` (a CatalogView or null) supplies unit
## roles and names. Returns {ok, error, view}.
static func from_payload(payload: Dictionary, catalog: RefCounted = null) -> Dictionary:
	for field in ["turn", "map_width", "civ", "tiles"]:
		if not payload.has(field):
			return _fail("visao sem campo: %s" % field)
	var width := int(payload["map_width"])
	if width <= 0:
		return _fail("largura do mapa invalida")
	if typeof(payload["tiles"]) != TYPE_ARRAY:
		return _fail("tiles nao e uma lista")
	var civ := int(payload["civ"])

	var max_index := -1
	for entry in payload["tiles"]:
		max_index = maxi(max_index, int(entry.get("tile", -1)))
	for entry in payload.get("cities", []):
		max_index = maxi(max_index, int(entry.get("city", {}).get("tile", -1)))
	for entry in payload.get("units", []):
		max_index = maxi(max_index, int(entry.get("unit", {}).get("tile", -1)))
	var height := maxi(int(payload.get("map_height", 0)), max_index / width + 1)
	if height <= 0:
		return _fail("mapa vazio")

	var grid: Array = []
	for row in height:
		var line: Array = []
		for col in width:
			line.append({"fog": "unknown"})
		grid.append(line)
	for entry in payload["tiles"]:
		var index := int(entry.get("tile", -1))
		if index < 0:
			return _fail("tile sem indice")
		var visibility := String(entry.get("visibility", ""))
		if visibility != "visible" and visibility != "remembered":
			continue  # "unknown" tiles are never sent with data; keep them blank
		var biome: Variant = entry.get("biome")
		if typeof(biome) == TYPE_STRING and BIOME_ASSETS.has(biome):
			biome = BIOME_ASSETS[biome]
		else:
			return _fail("bioma desconhecido: %s" % biome)
		var cell := Hex.cell_of_index(index, width)
		var tile: Dictionary = {"fog": visibility, "biome": biome, "river": bool(entry.get("river", false)), "yields": entry.get("yields", {})}
		if entry.get("improvement") != null:
			tile["improvement"] = String(entry["improvement"])
		if entry.has("build_progress") and entry["build_progress"] != null:
			tile["build_progress"] = entry["build_progress"]
		var owner: Variant = entry.get("owner")
		tile["owner"] = int(owner) if owner != null else -1
		grid[cell.y][cell.x] = tile
	_compute_borders(grid, width, height)

	var view := WorldView.new()
	view.live = true
	view.world_id = int(payload.get("world_id", 0))
	view.civ_id = civ
	view.turn = int(payload["turn"])
	view.map_width = width
	view.map_height = height
	view.tiles = grid
	view.era = ""
	view.civ_colors = {}
	view.civilization = {"id": civ, "name": "Civilização %d" % (civ + 1), "color": "#" + civ_color(civ).to_html(false)}
	var state: Dictionary = payload.get("civilization", {}) if typeof(payload.get("civilization")) == TYPE_DICTIONARY else {}
	_fill_civilization(view, state, payload)

	var first_own_city := Vector2i(-1, -1)
	for entry in payload.get("cities", []):
		var data: Dictionary = entry.get("city", {})
		var cell := Hex.cell_of_index(int(data.get("tile", 0)), width)
		var owner := int(data.get("owner", -1))
		var tile: Dictionary = grid[cell.y][cell.x]
		if tile["fog"] == "unknown":
			continue
		var queue: Array = []
		for unit_type in data.get("unit_queue", []):
			queue.append(String(unit_type))
		var yields: Variant = data.get("last_yields", {})
		var city := {
			"id": int(entry.get("id", 0)), "name": "", "cell": cell, "owner": owner, "own": owner == civ,
			"population": int(data.get("population", 0)), "housing": int(data.get("housing", 0)),
			"focus": String(data.get("focus", "")), "food_stock": int(data.get("food_stock", 0)),
			"stability": int(data.get("stability", 0)), "queue": queue,
			"unit_production": int(data.get("unit_production", 0)),
			"yields": yields if typeof(yields) == TYPE_DICTIONARY else {},
			"indicators": {"stability": int(data.get("stability", 0)), "deprivation": int(data.get("deprivation", 0)), "group_tension": int(data.get("group_tension", 0)), "war_threat": int(data.get("war_threat", 0)), "environmental_exposure": int(data.get("environmental_exposure", 0)), "crisis_pressure": int(data.get("crisis_pressure", 0))},
		}
		view.cities.append(city)
		tile["city"] = city
		if owner == civ and first_own_city.x < 0:
			first_own_city = cell
	_name_cities(view)
	var home: Variant = payload.get("home_tile")
	if (typeof(home) == TYPE_INT or typeof(home) == TYPE_FLOAT) and view.own_cities().is_empty():
		view.home_cell = Hex.cell_of_index(int(home), width)

	var width_i := width
	var order_info_present := false
	for entry in payload.get("units", []):
		var data: Dictionary = entry.get("unit", {})
		var cell := Hex.cell_of_index(int(data.get("tile", 0)), width_i)
		var type := String(data.get("unit_type", ""))
		var owner := int(data.get("owner", -1))
		var parsed := parse_order(data.get("order"), width_i)
		order_info_present = order_info_present or parsed["kind"] != ""
		var skipped: Variant = data.get("skipped_turn")
		var buildable: Array = []
		if owner == civ and typeof(data.get("buildable")) == TYPE_ARRAY:
			for improvement in data["buildable"]:
				if typeof(improvement) == TYPE_STRING:
					buildable.append(improvement)
		var info := _unit_info(catalog, type)
		view.units.append({
			"id": int(entry.get("id", 0)), "owner": owner, "own": owner == civ, "cell": cell, "type": type,
			"name": info["name"], "role": info["role"], "movement_max": info["movement"],
			"hp": int(data.get("hit_points", 0)), "movement_left": int(data.get("movement_left", 0)),
			"order": parsed["kind"], "order_target": parsed["target"], "order_improvement": parsed.get("improvement", ""),
			"skipped_turn": int(skipped) if skipped != null else -1, "buildable": buildable,
		})
		view.civ_colors[owner] = civ_color(owner)
	view.civ_colors[civ] = civ_color(civ)
	view.units.sort_custom(func(a: Dictionary, b: Dictionary) -> bool: return a["id"] < b["id"])

	# Server list is authoritative; keep only ids that are own units we can see.
	for unit_id in payload.get("idle_units", []):
		var unit := view.unit_by_id(int(unit_id))
		if not unit.is_empty() and unit["own"]:
			view.idle_units.append(int(unit_id))
	view.idle_units.sort()

	for entry in payload.get("pending_events", []):
		var card := _event_card(entry, view.turn, catalog)
		if not card.is_empty():
			view.pending_events.append({"id": int(entry.get("id", 0)), "template_id": card["title_id"], "choices": card["raw_choices"]})
			view.agenda.append(card["card"])
	view.agenda = WorldView._limit_agenda(view.agenda)

	view.capital = first_own_city
	if view.capital.x < 0 and view.home_cell.x >= 0:
		view.capital = view.home_cell
	if view.capital.x < 0:
		for unit in view.units:
			if unit["own"]:
				view.capital = unit["cell"]
				break
	if view.capital.x < 0:
		view.capital = Vector2i(width / 2, height / 2)
	return {"ok": true, "error": "", "view": view}


## The server sends no city names. Own cities: "Capital" then "Cidade 2", "Cidade 3"... by id order;
## foreign ones are named after their civilization.
static func _name_cities(view: RefCounted) -> void:
	var ordinal: Dictionary = {}
	view.cities.sort_custom(func(a: Dictionary, b: Dictionary) -> bool: return a["id"] < b["id"])
	for city in view.cities:
		var owner: int = city["owner"]
		ordinal[owner] = int(ordinal.get(owner, 0)) + 1
		var label := "Capital" if ordinal[owner] == 1 else "Cidade %d" % ordinal[owner]
		city["name"] = label if city["own"] else "%s da civilização %d" % [label, owner + 1]


static func _fill_civilization(view: RefCounted, state: Dictionary, payload: Dictionary) -> void:
	var wealth := int(state.get("treasury_wealth", 0))
	view.resources = {"riqueza": wealth, "conhecimento": int(state.get("knowledge", 0)), "cultura": int(state.get("culture", 0))}
	view.indicators = {"coesao": int(state.get("cohesion", 0)), "legitimidade": int(state.get("legitimacy", 0)), "deprivation": int(state.get("deprivation", 0)), "group_tension": int(state.get("group_tension", 0)), "war_threat": int(state.get("war_threat", 0)), "environmental_exposure": int(state.get("environmental_exposure", 0)), "crisis_pressure": int(state.get("crisis_pressure", 0))}
	view.header_chips = [
		{"icon": "riqueza", "value": wealth, "label": "Riqueza"},
		{"icon": "conhecimento", "value": int(state.get("knowledge", 0)), "label": "Conhecimento"},
		{"icon": "cultura", "value": int(state.get("culture", 0)), "label": "Cultura"},
		{"icon": "coesao", "value": int(state.get("cohesion", 0)), "label": "Coesão"},
	]
	var current: Variant = state.get("research")
	var done: Array = []
	for id in state.get("researched_technologies", []):
		done.append(String(id))
	var progress: Dictionary = {}
	var raw_progress: Variant = state.get("research_progress", {})
	if typeof(raw_progress) == TYPE_DICTIONARY:
		for id in raw_progress:
			progress[String(id)] = int(raw_progress[id])
	# Only the explicit contract rate can support an estimate. City knowledge yields are
	# not guaranteed to be the effective research rate (modifiers may apply server-side).
	var raw_per_turn: Variant = payload.get("research_per_turn")
	var per_turn := int(raw_per_turn) if typeof(raw_per_turn) == TYPE_INT or typeof(raw_per_turn) == TYPE_FLOAT else 0
	var percentages: Array = payload.get("research_percentages", [])
	var investment := int(state.get("research_investment", 0))
	view.research = {"current": String(current) if current != null else "", "done": done, "progress": progress, "per_turn": per_turn, "investment": investment, "percentages": percentages}


## Border edges: where the neighbor in that direction has another owner (or none). Display only.
static func _compute_borders(grid: Array, width: int, height: int) -> void:
	for row in height:
		for col in width:
			var tile: Dictionary = grid[row][col]
			if tile["fog"] == "unknown" or tile["owner"] < 0:
				continue
			var edges: Array = []
			var offsets := Hex.edge_offsets(row)
			for i in offsets.size():
				var candidate: Vector2i = Vector2i(col, row) + offsets[i]
				if not Hex.in_rows(candidate, height):
					continue
				var other: Dictionary = grid[candidate.y][Hex.wrap_col(candidate.x, width)]
				var other_owner: int = other["owner"] if other["fog"] != "unknown" else -1
				if other_owner != tile["owner"]:
					edges.append(Hex.EDGES[i])
			if not edges.is_empty():
				tile["borders"] = edges


static func _unit_info(catalog: RefCounted, type: String) -> Dictionary:
	var name := humanize(type)
	var role: String = ROLE_BY_TYPE.get(type, "")
	var movement := 0
	if catalog != null and catalog.unit_types.has(type):
		var entry: Dictionary = catalog.unit_types[type]
		name = String(entry.get("name", name))
		role = String(entry.get("role", role))
		movement = int(entry.get("movement", 0))
	return {"name": name, "role": role, "movement": movement}


## Turns a pending Entropy event into an agenda card. The optional catalog supplies readable text
## and PT-BR labels; ids/effects remain a resilient fallback during staggered server rollout.
static func _event_card(entry: Dictionary, turn: int, catalog: RefCounted = null) -> Dictionary:
	var event: Variant = entry.get("event")
	if typeof(event) != TYPE_DICTIONARY:
		return {}
	var event_id := int(entry.get("id", 0))
	var template := String(event.get("template_id", ""))
	var template_data: Dictionary = catalog.event_template(template) if catalog != null and catalog.has_method("event_template") else {}
	var category := String(template_data.get("category", event.get("category", "?")))
	var narrative := String(template_data.get("text", template_data.get("narrative", template_data.get("description", ""))))
	var template_choices: Dictionary = {}
	for template_choice in template_data.get("choices", []):
		if typeof(template_choice) == TYPE_DICTIONARY and typeof(template_choice.get("id")) == TYPE_STRING:
			template_choices[template_choice["id"]] = template_choice
	var options: Array = []
	var raw_choices: Array = event.get("choices", [])
	for choice in raw_choices:
		if typeof(choice) != TYPE_DICTIONARY:
			continue
		var choice_id := String(choice.get("id", ""))
		var template_choice: Dictionary = template_choices.get(choice_id, {})
		var label := String(template_choice.get("label", template_choice.get("name", choice.get("label", humanize(choice_id)))))
		options.append({"id": choice_id, "label": label, "sacrifice": _describe_effects(choice.get("effects", []))})
	var card := {
		"id": "event-%d" % event_id, "event_id": event_id, "severity": "important",
		"title": String(template_data.get("name", humanize(template))), "category": category,
		"cause": "Categoria: %s" % category_label(category), "effect": narrative, "risk": "", "deadline_turns": maxi(0, int(event.get("expires", turn)) - turn), "options": options,
	}
	return {"card": card, "title_id": template, "raw_choices": raw_choices}


const CATEGORY_LABELS := {
	"climate": "clima", "collapse": "colapso", "diplomacy": "diplomacia", "discovery": "descoberta",
	"epidemic": "epidemia", "interference": "interferência", "magic": "magia", "renewal": "renovação",
	"revolt": "revolta", "social": "social", "technology": "tecnologia", "terrain": "terreno",
}
const RESOURCE_LABELS := {
	"food": "comida", "production": "produção", "wealth": "riqueza", "knowledge": "conhecimento", "culture": "cultura",
}


static func category_label(category: String) -> String:
	return CATEGORY_LABELS.get(category, humanize(category) if not category.is_empty() else "evento")


static func resource_label(resource: String) -> String:
	return RESOURCE_LABELS.get(resource, humanize(resource))


## PT-BR names of the event catalog tags (engine ids are never shown raw when known).
const TAG_LABELS := {
	"autonomy.emergency": "autonomia de emergência",
	"autonomy.negotiated": "autonomia negociada",
	"biome.forest": "floresta",
	"biome.plains": "planície",
	"border.disputed": "fronteira disputada",
	"border.sanctuary_claim": "santuário reivindicado",
	"cism.forest_cult": "cisma do culto",
	"cism.tolerated": "cisma tolerado",
	"claim.metal": "veio reivindicado",
	"crisis.administration_fractured": "administração fragmentada",
	"diplomacy.misread_envoy": "mal-entendido diplomático",
	"discovery.preservation_workshop": "oficina de conservação",
	"entropy.appeasement": "rito de apaziguamento",
	"entropy.appeasement_requested": "apaziguamento pedido",
	"entropy.diversion_researched": "desvio pesquisado",
	"entropy.forecast": "presságio",
	"entropy.forecast_requested": "presságio pedido",
	"event.drought": "seca",
	"group.tension": "tensão de grupos",
	"growth.suspended": "crescimento suspenso",
	"harvest.preserved": "colheita protegida",
	"health.water_risk": "risco na água",
	"ledger.offense": "ofensa registrada",
	"ledger.recent_offense": "ofensa recente",
	"magic.ember_rain": "chuva de brasas",
	"magic.moonlit_tide": "maré do luar",
	"magic.whispering_grove": "bosque dos sussurros",
	"repression.cism": "repressão ao cisma",
	"repression.guard": "guarda imposta",
	"research.forbidden": "pesquisa proibida",
	"resource.metal_revealed": "metal revelado",
	"revolt.assembly": "assembleia em revolta",
	"revolt.tribute_refused": "tributo recusado",
	"route.coastal": "rota costeira",
	"route.crossing": "travessia",
	"route.flooded": "rota alagada",
	"route.rerouted": "rota desviada",
	"storage.drawn": "estoques usados",
	"storage.prepared": "estoques preparados",
	"terrain.new_bank_settlement": "nova margem ocupada",
	"terrain.prospectable": "terreno prospectável",
	"terrain.protected": "terreno protegido",
	"terrain.river_shifted": "rio desviado",
	"trade.held": "comércio retido",
	"trade.quarantined": "comércio em quarentena",
}


static func _describe_effects(effects: Array) -> String:
	var parts: Array[String] = []
	for effect in effects:
		if typeof(effect) != TYPE_DICTIONARY:
			continue
		match String(effect.get("op", "")):
			"adjust_resource":
				parts.append("%s %+d" % [resource_label(String(effect.get("resource", "?"))), int(effect.get("amount", 0))])
			"add_tag", "remove_tag":
				var tag := String(effect.get("tag", effect.get("tag_id", "")))
				parts.append(("+ " if effect.get("op") == "add_tag" else "− ") + String(TAG_LABELS.get(tag, humanize(tag))))
			_:
				parts.append(humanize(String(effect.get("op", ""))))
	return ", ".join(parts)


static func _fail(message: String) -> Dictionary:
	return {"ok": false, "error": message, "view": null}

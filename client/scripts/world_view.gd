extends RefCounted
## Client-side projection of the server state snapshot (type "state_snapshot").
## The client never computes game rules; this only validates and exposes what the server sent.

const Protocol := preload("res://scripts/protocol.gd")

const MAX_CRITICAL := 1
const MAX_IMPORTANT := 2
const KNOWN_FOG: Array[String] = ["visible", "remembered", "unknown"]

var turn := 0
var era := ""
var civilization: Dictionary = {}
var resources: Dictionary = {}
var indicators: Dictionary = {}
var ready := false
var agenda: Array = []
var map_width := 0
var map_height := 0
var capital := Vector2i.ZERO
var tiles: Array = []  # tiles[row][col] -> Dictionary

# Live-server fields (filled by server_view.gd; empty for the hand-written fixture).
var live := false
var world_id := 0
var civ_id := 0
## Resource chips shown in the header: Array of {icon, value, label}. Empty = use `resources`.
var header_chips: Array = []
## Units visible to this civilization. Each: {id, owner, own, cell, type, role, hp, movement_left,
## order ("Idle"|"Fortify"|"Explore"|"MoveTo"|"Sentry"|"" when the server did not say), order_target, skipped_turn}.
var units: Array = []
## Ids of own units awaiting an order (server list, adjusted by local optimistic orders).
var idle_units: Array = []
## Entropy events waiting for a response: Array of {id, template_id, choices}.
var pending_events: Array = []
## Research state of the own civilization: {current: String, done: Array, progress: Dictionary}.
var research: Dictionary = {"current": "", "done": [], "progress": {}, "per_turn": 0, "investment": 0, "percentages": []}
var civ_colors: Dictionary = {}  # civ id -> Color
## Starting tile of the civilization, only while it has no city (`home_tile`); (-1, -1) otherwise.
var home_cell := Vector2i(-1, -1)
## Cities seen by this civilization. Each: {id, name, cell, owner, own, population, housing, focus,
## food_stock, stability, queue: Array[String], unit_production, yields: {food, production, ...}}.
var cities: Array = []
## Relations with known civilizations (C2b): one entry per civilization the server has put this one
## in contact with. Each: {civ, name, color, state, confidence, resentment, debt, ledger: Array}.
## Empty against a server that does not send the `relations` field yet.
var relations: Array = []
## Cells where a founding was accepted this turn but the city does not exist yet.
var founding_cells: Array = []


## "N unidades aguardam ordem" copy shared by the Pauta and the tests.
static func gate_text(count: int) -> String:
	if count == 1:
		return "1 unidade aguarda ordem"
	return "%d unidades aguardam ordem" % count


func city_by_id(city_id: int) -> Dictionary:
	for city in cities:
		if city["id"] == city_id:
			return city
	return {}


func city_at(cell: Vector2i) -> Dictionary:
	for city in cities:
		if city["cell"] == cell:
			return city
	return {}


func own_cities() -> Array:
	return cities.filter(func(city: Dictionary) -> bool: return city["own"])


## True when the Pauta must offer "Fundar a capital": the server named a starting tile and no own city exists.
func needs_capital() -> bool:
	return live and home_cell.x >= 0 and own_cities().is_empty()


func is_founding(cell: Vector2i) -> bool:
	return founding_cells.has(cell)


## Optimistic update after the server accepted FoundCity: the city appears when the turn resolves.
## A settler on the tile is consumed then, so it stops waiting for an order.
func apply_local_found(cell: Vector2i) -> void:
	if not founding_cells.has(cell):
		founding_cells.append(cell)
	for unit in units:
		if unit["own"] and unit["cell"] == cell and unit["type"] == "unit.settler":
			unit["founding"] = true
			unit["skipped_turn"] = turn
			idle_units.erase(unit["id"])


func apply_local_queue(city_id: int, unit_type: String) -> void:
	var city := city_by_id(city_id)
	if not city.is_empty():
		city["queue"].append(unit_type)


## Optimistic update after RemoveQueuedUnit: the item goes, the accumulated production stays.
func apply_local_queue_remove(city_id: int, index: int) -> void:
	var city := city_by_id(city_id)
	if not city.is_empty() and index >= 0 and index < city["queue"].size():
		city["queue"].remove_at(index)


## Optimistic update after MoveQueuedUnit: the item at `from` ends at position `to`.
func apply_local_queue_move(city_id: int, from: int, to: int) -> void:
	var city := city_by_id(city_id)
	if city.is_empty():
		return
	var queue: Array = city["queue"]
	if from < 0 or to < 0 or from >= queue.size() or to >= queue.size():
		return
	var item: Variant = queue[from]
	queue.remove_at(from)
	queue.insert(to, item)


func apply_local_focus(city_id: int, focus: String) -> void:
	var city := city_by_id(city_id)
	if not city.is_empty():
		city["focus"] = focus


## Optimistic display only after SetResearch is accepted; the next projection is authoritative.
func apply_local_research(research_id: String) -> void:
	research["current"] = research_id


## Optimistic display only after SetResearchInvestment is accepted.
func apply_local_research_investment(percent: int) -> void:
	research["investment"] = percent
	research["investment_local"] = true  # the projected rate still belongs to the old percentage


## Remaining decisions in the stable order the compact Pronto button walks: the capital card first,
## then agenda cards (events) not yet answered in `answered` (card ids), then idle units by id.
## Each entry: {kind: "capital"} | {kind: "event", id: String} | {kind: "unit", id: int}.
func decisions(answered: Array = []) -> Array:
	var list: Array = []
	if needs_capital() and not is_founding(home_cell):
		list.append({"kind": "capital"})
	for card in agenda:
		if not answered.has(card["id"]):
			list.append({"kind": "event", "id": card["id"]})
	var ids: Array = idle_units.duplicate()
	ids.sort()
	for unit_id in ids:
		list.append({"kind": "unit", "id": unit_id})
	return list


func awaiting_count() -> int:
	return idle_units.size()


## True while at least one own idle unit blocks the "Pronto" button (docs/sdd/10-protocolo.md §5.1).
func ready_blocked() -> bool:
	return not idle_units.is_empty()


func unit_by_id(unit_id: int) -> Dictionary:
	for unit in units:
		if unit["id"] == unit_id:
			return unit
	return {}


## Units standing on `cell`, own first, in id order.
func units_at(cell: Vector2i) -> Array:
	var result: Array = []
	for unit in units:
		if unit["cell"] == cell:
			result.append(unit)
	result.sort_custom(func(a: Dictionary, b: Dictionary) -> bool:
		if a["own"] != b["own"]:
			return a["own"]
		return a["id"] < b["id"])
	return result


## Optimistic local update after the server accepted a command for the open turn.
## The authoritative `idle_units` list replaces this on the next snapshot.
func apply_local_order(unit_id: int, order_kind: String, order_target: Vector2i = Vector2i(-1, -1)) -> void:
	var unit := unit_by_id(unit_id)
	if unit.is_empty():
		return
	unit["order"] = order_kind
	unit["order_target"] = order_target
	if order_kind == "Idle":
		if unit["movement_left"] > 0 and not idle_units.has(unit_id):
			idle_units.append(unit_id)
			idle_units.sort()
	else:
		idle_units.erase(unit_id)


func apply_local_build(unit_id: int, improvement: String) -> void:
	var unit := unit_by_id(unit_id)
	if unit.is_empty():
		return
	unit["order"] = "Build"
	unit["order_improvement"] = improvement
	idle_units.erase(unit_id)


func apply_local_attack(unit_id: int) -> void:
	var unit := unit_by_id(unit_id)
	if not unit.is_empty():
		unit["attacked_turn"] = turn
	idle_units.erase(unit_id)


func apply_local_skip(unit_id: int) -> void:
	var unit := unit_by_id(unit_id)
	if unit.is_empty():
		return
	unit["skipped_turn"] = turn
	idle_units.erase(unit_id)


## Builds a WorldView from an envelope Dictionary (output of Protocol.parse_envelope).
## Returns {ok, error, view}.
static func from_envelope(envelope: Dictionary) -> Dictionary:
	if envelope.get("type") != "state_snapshot":
		return {"ok": false, "error": "tipo inesperado: %s" % envelope.get("type"), "view": null}
	return from_payload(envelope["payload"])


static func from_payload(payload: Dictionary) -> Dictionary:
	var view := new()
	for field in ["turn", "era", "civilization", "resources", "agenda", "map"]:
		if not payload.has(field):
			return {"ok": false, "error": "snapshot sem campo: %s" % field, "view": null}
	view.turn = int(payload["turn"])
	view.era = String(payload["era"])
	view.civilization = payload["civilization"]
	view.resources = payload["resources"]
	view.indicators = payload.get("indicators", {})
	view.ready = bool(payload.get("ready", false))
	view.agenda = _limit_agenda(payload["agenda"])
	var map: Dictionary = payload["map"]
	view.map_width = int(map.get("width", 0))
	view.map_height = int(map.get("height", 0))
	var cap: Array = map.get("capital", [0, 0])
	view.capital = Vector2i(int(cap[0]), int(cap[1]))
	view.tiles = map.get("tiles", [])
	if view.map_width <= 0 or view.map_height <= 0 or view.tiles.size() != view.map_height:
		return {"ok": false, "error": "mapa com dimensoes invalidas", "view": null}
	for row in view.tiles:
		if row.size() != view.map_width:
			return {"ok": false, "error": "linha do mapa com largura errada", "view": null}
		for tile in row:
			if not KNOWN_FOG.has(tile.get("fog", "")):
				return {"ok": false, "error": "tile com fog invalido", "view": null}
	return {"ok": true, "error": "", "view": view}


## GDD 11: at most one critical and two important decisions per agenda, critical first.
static func _limit_agenda(cards: Array) -> Array:
	var result: Array = []
	var critical := 0
	var important := 0
	for card in cards:
		if card.get("severity") == "critical" and critical < MAX_CRITICAL:
			critical += 1
			result.append(card)
		elif card.get("severity") == "important" and important < MAX_IMPORTANT:
			important += 1
			result.append(card)
	result.sort_custom(func(a: Dictionary, b: Dictionary) -> bool: return a["severity"] == "critical" and b["severity"] != "critical")
	return result


func tile_at(cell: Vector2i) -> Dictionary:
	return tiles[cell.y][cell.x]


static func load_fixture(path: String) -> Dictionary:
	var file := FileAccess.open(path, FileAccess.READ)
	if file == null:
		return {"ok": false, "error": "nao foi possivel abrir %s" % path, "view": null}
	var parsed := Protocol.parse_envelope(file.get_as_text())
	if not parsed["ok"]:
		return {"ok": false, "error": parsed["error"], "view": null}
	return from_envelope(parsed["envelope"])

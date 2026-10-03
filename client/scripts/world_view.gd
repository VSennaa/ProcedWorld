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

extends RefCounted
## Protocol client helpers (docs/sdd/10-protocolo.md): JSON envelope parsing and building.
## No networking here, so it is fully testable headless.

const PROTOCOL_VERSION := "1.0"
const SUPPORTED_MAJOR := 1


## Parses an envelope from JSON text. Returns {ok, error, envelope}.
## The envelope has protocol_version, request_id, type and payload. Unknown fields are ignored.
static func parse_envelope(text: String) -> Dictionary:
	var json := JSON.new()
	if json.parse(text) != OK:
		return _fail("JSON invalido: %s" % json.get_error_message())
	return validate_envelope(json.data)


static func validate_envelope(data: Variant) -> Dictionary:
	if typeof(data) != TYPE_DICTIONARY:
		return _fail("envelope nao e um objeto")
	var version: Variant = data.get("protocol_version")
	# Servers older than the "1.0" text version sent the bare integer 1.
	if typeof(version) == TYPE_INT or (typeof(version) == TYPE_FLOAT and version == floorf(version)):
		version = "%d.0" % int(version)
	if typeof(version) != TYPE_STRING:
		return _fail("protocol_version ausente")
	if not is_version_compatible(version):
		return _fail("versao incompativel: %s (suportada: %s)" % [version, PROTOCOL_VERSION])
	if typeof(data.get("type")) != TYPE_STRING or String(data["type"]).is_empty():
		return _fail("campo obrigatorio ausente: type")
	# Unsolicited pushes (turn_diff, catalog, ...) carry request_id null; the key must still exist.
	if not data.has("request_id") or (typeof(data["request_id"]) != TYPE_STRING and data["request_id"] != null):
		return _fail("campo obrigatorio ausente: request_id")
	if typeof(data["request_id"]) == TYPE_STRING and String(data["request_id"]).is_empty():
		return _fail("campo obrigatorio ausente: request_id")
	if typeof(data.get("payload")) != TYPE_DICTIONARY:
		return _fail("payload ausente ou nao e um objeto")
	return {
		"ok": true,
		"error": "",
		"envelope": {
			"protocol_version": version,
			"request_id": "" if data["request_id"] == null else data["request_id"],
			"type": data["type"],
			"payload": data["payload"],
		},
	}


## Same major version is compatible; a newer minor only adds optional fields.
static func is_version_compatible(version: String) -> bool:
	var parts := version.split(".")
	return parts.size() >= 2 and parts[0].is_valid_int() and parts[1].is_valid_int() and parts[0].to_int() == SUPPORTED_MAJOR


static func build_envelope(type: String, payload: Dictionary, request_id: String) -> String:
	return JSON.stringify({
		"protocol_version": PROTOCOL_VERSION,
		"request_id": request_id,
		"type": type,
		"payload": payload,
	})


# --- Command payloads for `submit_command` (serde adjacent tagging: {"type": snake_case, "data": {...}}) ---

## UnitOrder on the wire (engine serde adjacent tagging): {"type": "fortify"}, {"type": "sentry"} or
## {"type": "move_to", "data": {"target": N}}. `kind` is the client name ("Idle", "MoveTo", ...).
const ORDER_WIRE := {"Idle": "idle", "Fortify": "fortify", "Explore": "explore", "MoveTo": "move_to", "Sentry": "sentry", "Build": "build"}


static func order_wire(kind: String, tile_index: int = -1) -> Dictionary:
	if kind == "MoveTo":
		return {"type": "move_to", "data": {"target": tile_index}}
	return {"type": ORDER_WIRE.get(kind, "idle")}


static func order_build(improvement: String) -> Dictionary:
	return {"type": "build", "data": {"improvement": improvement}}


static func command_declare_attack(attacker: int, target: int) -> Dictionary:
	return {"command": {"type": "declare_attack", "data": {"attacker": attacker, "target": target}}}


static func order_move_to(tile_index: int) -> Dictionary:
	return order_wire("MoveTo", tile_index)


static func command_set_unit_order(unit_id: int, order: Variant) -> Dictionary:
	return {"command": {"type": "set_unit_order", "data": {"unit_id": unit_id, "order": order}}}


static func command_skip_unit(unit_id: int) -> Dictionary:
	return {"command": {"type": "skip_unit", "data": {"unit_id": unit_id}}}


## City ids are chosen by the client (docs/sdd/10 sec. 6.1): the first city of a civilization uses
## the civilization id; a city founded by a settler uses SETTLER_CITY_ID_BASE + the settler's unit id
## (the same rule the bots follow). A taken id comes back as a clean `command_rejected`.
const SETTLER_CITY_ID_BASE := 1000000


static func settler_city_id(unit_id: int) -> int:
	return SETTLER_CITY_ID_BASE + unit_id


static func command_found_city(city_id: int, tile_index: int) -> Dictionary:
	return {"command": {"type": "found_city", "data": {"city_id": city_id, "target": tile_index}}}


static func command_queue_unit(city_id: int, unit_type: String) -> Dictionary:
	return {"command": {"type": "queue_unit", "data": {"city_id": city_id, "unit_type": unit_type}}}


static func command_remove_queued_unit(city_id: int, index: int) -> Dictionary:
	return {"command": {"type": "remove_queued_unit", "data": {"city_id": city_id, "index": index}}}


## Moves the queue item at `from` so that it ends at position `to`.
static func command_move_queued_unit(city_id: int, from: int, to: int) -> Dictionary:
	return {"command": {"type": "move_queued_unit", "data": {"city_id": city_id, "from": from, "to": to}}}


## `focus` is the engine CityFocus name: "supply", "build" or "diversify".
static func command_set_city_focus(city_id: int, focus: String) -> Dictionary:
	return {"command": {"type": "set_city_focus", "data": {"city_id": city_id, "focus": focus}}}


static func command_set_research(research: String) -> Dictionary:
	return {"command": {"type": "set_research", "data": {"research": research}}}


static func command_set_research_investment(percent: int) -> Dictionary:
	return {"command": {"type": "set_research_investment", "data": {"percent": percent}}}


static func command_respond_to_event(event_id: int, choice_id: String) -> Dictionary:
	return {"command": {"type": "respond_to_event", "data": {"event_id": event_id, "choice_id": choice_id}}}


static func _fail(message: String) -> Dictionary:
	return {"ok": false, "error": message, "envelope": {}}

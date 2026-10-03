extends RefCounted
## Client view of the server's `catalog` frame (docs/sdd/10-protocolo.md §5.2): tech tree and unit
## types. Read-only; the client never reads data/ catalogs directly.
##
## Assumed payload shape (the exact field names belong to the server task):
##   {catalog_version, catalog_hash,
##    technologies: [{id, name, cost, prerequisites: [id], branch?}],
##    unit_types:   [{id, name, role, movement, strength}]}
## `tech_tree` / `units` are accepted as aliases of the two lists.

var version := 0
var hash_text := ""
var techs: Dictionary = {}       # id -> {id, name, cost, prerequisites, branch}
var tech_order: Array[String] = []  # ids in the order the server listed them
var unit_types: Dictionary = {}  # id -> {id, name, role, movement, strength}
var _depth: Dictionary = {}


## Returns {ok, error, catalog}. Rejects duplicate ids and prerequisite cycles.
static func from_payload(payload: Dictionary) -> Dictionary:
	var catalog := new()
	catalog.version = int(payload.get("catalog_version", payload.get("version", 0)))
	catalog.hash_text = String(payload.get("catalog_hash", payload.get("hash", "")))
	var tech_list: Variant = payload.get("technologies", payload.get("tech_tree", []))
	if typeof(tech_list) == TYPE_DICTIONARY:
		tech_list = tech_list.get("technologies", [])
	if typeof(tech_list) != TYPE_ARRAY:
		return _fail("catalogo sem lista de tecnologias")
	for entry in tech_list:
		if typeof(entry) != TYPE_DICTIONARY or typeof(entry.get("id")) != TYPE_STRING:
			return _fail("tecnologia sem id")
		var id: String = entry["id"]
		if catalog.techs.has(id):
			return _fail("tecnologia duplicada: %s" % id)
		var prereqs: Array[String] = []
		for p in entry.get("prerequisites", entry.get("requires", [])):
			prereqs.append(String(p))
		catalog.techs[id] = {
			"id": id, "name": String(entry.get("name", id)), "cost": int(entry.get("cost", 0)),
			"prerequisites": prereqs, "branch": String(entry.get("branch", "")),
		}
		catalog.tech_order.append(id)
	var unit_list: Variant = payload.get("unit_types", payload.get("units", []))
	if typeof(unit_list) == TYPE_ARRAY:
		for entry in unit_list:
			if typeof(entry) == TYPE_DICTIONARY and typeof(entry.get("id")) == TYPE_STRING:
				catalog.unit_types[entry["id"]] = entry
	for id in catalog.techs:
		if catalog._has_cycle(id, {}):
			return _fail("ciclo de pre-requisitos em %s" % id)
	return {"ok": true, "error": "", "catalog": catalog}


static func load_fixture(path: String) -> Dictionary:
	var file := FileAccess.open(path, FileAccess.READ)
	if file == null:
		return _fail("nao foi possivel abrir %s" % path)
	var data: Variant = JSON.parse_string(file.get_as_text())
	if typeof(data) != TYPE_DICTIONARY:
		return _fail("catalogo invalido")
	return from_payload(data)


func tech_name(id: String) -> String:
	return String(techs[id]["name"]) if techs.has(id) else id


## Depth = longest prerequisite chain below the tech (0 for roots). Unknown prerequisites are ignored.
func depth(id: String) -> int:
	if _depth.has(id):
		return _depth[id]
	var best := 0
	for p in techs[id]["prerequisites"]:
		if techs.has(p):
			best = maxi(best, depth(p) + 1)
	_depth[id] = best
	return best


## Columns by prerequisite depth: Array of Array[String] (tech ids), each sorted by branch then name.
func columns() -> Array:
	var result: Array = []
	for id in tech_order:
		var d := depth(id)
		while result.size() <= d:
			result.append([])
		result[d].append(id)
	for column in result:
		column.sort_custom(func(a: String, b: String) -> bool:
			var ta: Dictionary = techs[a]
			var tb: Dictionary = techs[b]
			if ta["branch"] != tb["branch"]:
				return ta["branch"] < tb["branch"]
			return ta["name"] < tb["name"])
	return result


## "done" | "current" | "available" | "locked" for the own civilization's research state
## ({current, done, progress}, see WorldView.research). Available = every prerequisite is done.
func status(id: String, research: Dictionary) -> String:
	if research.get("done", []).has(id):
		return "done"
	if research.get("current", "") == id:
		return "current"
	for p in techs[id]["prerequisites"]:
		if techs.has(p) and not research.get("done", []).has(p):
			return "locked"
	return "available"


## Progress {have, cost} toward a tech; a finished one reads as full.
func progress(id: String, research: Dictionary) -> Dictionary:
	var cost: int = techs[id]["cost"]
	if research.get("done", []).has(id):
		return {"have": cost, "cost": cost}
	return {"have": mini(int(research.get("progress", {}).get(id, 0)), cost), "cost": cost}


## Names of prerequisites still missing.
func missing_prerequisites(id: String, research: Dictionary) -> Array[String]:
	var result: Array[String] = []
	for p in techs[id]["prerequisites"]:
		if techs.has(p) and not research.get("done", []).has(p):
			result.append(tech_name(p))
	return result


func _has_cycle(id: String, visiting: Dictionary) -> bool:
	if visiting.has(id):
		return true
	visiting[id] = true
	for p in techs[id]["prerequisites"]:
		if techs.has(p) and _has_cycle(p, visiting):
			return true
	visiting.erase(id)
	return false


static func _fail(message: String) -> Dictionary:
	return {"ok": false, "error": message, "catalog": null}

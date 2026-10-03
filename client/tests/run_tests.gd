extends SceneTree
## Headless tests: godot --headless --path client -s res://tests/run_tests.gd
## No network, no scene tree needed beyond this script.

const Hex := preload("res://scripts/hex.gd")
const Protocol := preload("res://scripts/protocol.gd")
const WorldView := preload("res://scripts/world_view.gd")
const ServerView := preload("res://scripts/server_view.gd")
const CatalogView := preload("res://scripts/catalog_view.gd")
const SessionStore := preload("res://scripts/session_store.gd")
const AgendaView := preload("res://scripts/agenda_view.gd")
const UnitPanel := preload("res://scripts/unit_panel.gd")
const CityPanel := preload("res://scripts/city_panel.gd")
const TechView := preload("res://scripts/tech_view.gd")
const ConnectView := preload("res://scripts/connect_view.gd")

const SERVER_VIEW := "res://fixtures/server_view.json"
const SERVER_CATALOG := "res://fixtures/server_catalog.json"

var _failures := 0
var _checks := 0


func _init() -> void:
	test_hex_neighbors()
	test_distance_and_wrap()
	test_center_and_pixel_to_cell()
	test_envelope()
	test_fixture()
	test_agenda_limits()
	test_push_envelopes()
	test_commands()
	test_hex_index()
	test_server_view_adapter()
	test_adapter_tolerates_missing_fields()
	test_adapter_rejects_bad_input()
	test_idle_queue_and_gate()
	test_agenda_ready_gate()
	test_unit_panel_actions()
	test_found_and_produce_model()
	test_capital_card()
	test_city_panel()
	test_found_and_queue_commands()
	test_catalog_layout()
	test_catalog_rejects_bad_input()
	test_session_store()
	test_views_build()
	print("%d checks, %d failures" % [_checks, _failures])
	if _failures == 0:
		print("ALL TESTS PASSED")
	quit(0 if _failures == 0 else 1)


func check(condition: bool, label: String) -> void:
	_checks += 1
	if not condition:
		_failures += 1
		printerr("FAIL: %s" % label)


func test_hex_neighbors() -> void:
	var even := Hex.neighbors(Vector2i(2, 2), 10, 10)
	check(even == [Vector2i(2, 1), Vector2i(3, 2), Vector2i(2, 3), Vector2i(1, 3), Vector2i(1, 2), Vector2i(1, 1)], "even-row neighbors in clockwise order")
	var odd := Hex.neighbors(Vector2i(2, 3), 10, 10)
	check(odd == [Vector2i(3, 2), Vector2i(3, 3), Vector2i(3, 4), Vector2i(2, 4), Vector2i(1, 3), Vector2i(2, 2)], "odd-row neighbors in clockwise order")
	var edge := Hex.neighbors(Vector2i(0, 4), 10, 10)
	check(edge.has(Vector2i(9, 4)) and edge.has(Vector2i(9, 3)), "neighbors wrap across the seam")
	check(Hex.neighbors(Vector2i(3, 0), 10, 10).size() == 4, "no neighbors beyond the pole")
	for cell in [Vector2i(0, 0), Vector2i(5, 5), Vector2i(9, 9)]:
		for n in Hex.neighbors(cell, 10, 10):
			check(Hex.neighbors(n, 10, 10).has(cell), "neighbor relation is symmetric %s-%s" % [cell, n])


func test_distance_and_wrap() -> void:
	check(Hex.distance(Vector2i(2, 2), Vector2i(2, 2)) == 0, "distance to self")
	check(Hex.distance(Vector2i(2, 2), Vector2i(3, 2)) == 1, "distance to east neighbor")
	check(Hex.distance(Vector2i(0, 0), Vector2i(4, 0)) == 4, "straight distance")
	check(Hex.distance(Vector2i(0, 0), Vector2i(0, 4)) == 4, "vertical distance")
	check(Hex.wrap_distance(Vector2i(0, 3), Vector2i(9, 3), 10) == 1, "wrap distance across the seam")
	check(Hex.wrap_distance(Vector2i(1, 3), Vector2i(8, 3), 10) == 3, "wrap distance shorter than direct")
	check(Hex.wrap_distance(Vector2i(0, 0), Vector2i(5, 0), 10) == 5, "wrap distance at antipode")
	check(Hex.wrap_col(-1, 10) == 9 and Hex.wrap_col(10, 10) == 0, "wrap_col")


func test_center_and_pixel_to_cell() -> void:
	var radius := 64.0
	var width := 12
	var height := 8
	for row in height:
		for col in width:
			var cell := Vector2i(col, row)
			var c := Hex.center(cell, radius)
			check(Hex.pixel_to_cell(c, radius, width, height) == cell, "center round-trips %s" % cell)
			check(Hex.pixel_to_cell(c + Vector2(20, 15), radius, width, height) == cell, "inside-hex offset %s" % cell)
	check(Hex.center(Vector2i(1, 1), radius).is_equal_approx(Vector2(radius * Hex.SQRT3 * 1.5, radius * 1.5)), "odd rows shift half a hex")
	var wrapped := Hex.center(Vector2i(-1, 2), radius)
	check(Hex.pixel_to_cell(wrapped, radius, width, height) == Vector2i(width - 1, 2), "pixel left of map wraps to last column")
	var beyond := Hex.center(Vector2i(width, 3), radius) + Vector2(0, 0)
	check(Hex.pixel_to_cell(beyond, radius, width, height) == Vector2i(0, 3), "pixel right of map wraps to column 0")
	check(Hex.pixel_to_cell(Vector2(100, -500), radius, width, height) == Vector2i(-1, -1), "above the map is invalid")
	check(Hex.pixel_to_cell(Vector2(100, 5000), radius, width, height) == Vector2i(-1, -1), "below the map is invalid")


func test_envelope() -> void:
	var good := Protocol.parse_envelope('{"protocol_version":"1.0","request_id":"r1","type":"ping","payload":{"a":1},"extra":true}')
	check(good["ok"] and good["envelope"]["type"] == "ping" and good["envelope"]["payload"]["a"] == 1, "valid envelope parses, unknown field ignored")
	check(Protocol.parse_envelope('{"protocol_version":"1.7","request_id":"r","type":"t","payload":{}}')["ok"], "newer minor version is accepted")
	var bad_major := Protocol.parse_envelope('{"protocol_version":"2.0","request_id":"r","type":"t","payload":{}}')
	check(not bad_major["ok"] and "incompativel" in bad_major["error"], "major version mismatch rejected")
	check(not Protocol.parse_envelope('{"request_id":"r","type":"t","payload":{}}')["ok"], "missing version rejected")
	check(not Protocol.parse_envelope('{"protocol_version":"1.0","type":"t","payload":{}}')["ok"], "missing request_id rejected")
	check(not Protocol.parse_envelope('{"protocol_version":"1.0","request_id":"r","type":"t"}')["ok"], "missing payload rejected")
	check(not Protocol.parse_envelope("not json")["ok"], "garbage rejected")
	check(not Protocol.parse_envelope("[1,2]")["ok"], "non-object rejected")
	var built := Protocol.parse_envelope(Protocol.build_envelope("ready_set", {"turn": 3}, "c-1"))
	check(built["ok"] and built["envelope"]["request_id"] == "c-1", "built envelope round-trips")


func test_fixture() -> void:
	var loaded := WorldView.load_fixture("res://fixtures/state_snapshot.json")
	check(loaded["ok"], "fixture loads: %s" % loaded["error"])
	if not loaded["ok"]:
		return
	var view = loaded["view"]
	check(view.turn == 42 and view.civilization["name"] == "Aurelia", "fixture header fields")
	check(view.map_width == 24 and view.map_height == 20 and view.tiles.size() == 20, "fixture map dimensions")
	check(view.tile_at(view.capital).has("city"), "capital tile holds a city")
	check(view.tile_at(view.capital)["fog"] == "visible", "capital is visible")
	var unknown_without_data := true
	var seen_unknown := false
	for row in view.tiles:
		for tile in row:
			if tile["fog"] == "unknown":
				seen_unknown = true
				if tile.has("biome") or tile.has("resource"):
					unknown_without_data = false
	check(seen_unknown and unknown_without_data, "unknown tiles carry no information")
	var bad := WorldView.from_envelope({"type": "state_snapshot", "payload": {"turn": 1}})
	check(not bad["ok"], "incomplete snapshot rejected")
	var wrong_type := WorldView.from_envelope({"type": "pong", "payload": {}})
	check(not wrong_type["ok"], "wrong message type rejected")


func test_agenda_limits() -> void:
	var loaded := WorldView.load_fixture("res://fixtures/state_snapshot.json")
	var view = loaded["view"]
	check(view.agenda.size() == 3 and view.agenda[0]["severity"] == "critical", "fixture agenda: 1 critical first, 2 important")
	var cards: Array = []
	for i in 3:
		cards.append({"id": "c%d" % i, "severity": "critical"})
		cards.append({"id": "i%d" % i, "severity": "important"})
	var file := FileAccess.open("res://fixtures/state_snapshot.json", FileAccess.READ)
	var payload: Dictionary = JSON.parse_string(file.get_as_text())["payload"]
	payload["agenda"] = cards
	var limited := WorldView.from_payload(payload)
	check(limited["ok"] and limited["view"].agenda.size() == 3, "agenda trimmed to 1 critical + 2 important")


# --- live client (P17) ---

func _load_view_payload() -> Dictionary:
	var file := FileAccess.open(SERVER_VIEW, FileAccess.READ)
	return JSON.parse_string(file.get_as_text())


func _load_catalog() -> RefCounted:
	var loaded := CatalogView.load_fixture(SERVER_CATALOG)
	check(loaded["ok"], "catalog fixture loads: %s" % loaded["error"])
	return loaded["catalog"]


func _adapt(payload: Dictionary, catalog: RefCounted = null) -> RefCounted:
	var result := ServerView.from_payload(payload, catalog)
	check(result["ok"], "adapter accepts payload: %s" % result["error"])
	return result["view"]


func test_push_envelopes() -> void:
	var push := Protocol.parse_envelope('{"protocol_version":"1.0","request_id":null,"type":"turn_diff","payload":{}}')
	check(push["ok"] and push["envelope"]["request_id"] == "", "push frame with request_id null parses")
	var legacy := Protocol.parse_envelope('{"protocol_version":1,"request_id":"r","type":"joined","payload":{}}')
	check(legacy["ok"] and legacy["envelope"]["protocol_version"] == "1.0", "integer protocol_version 1 is read as 1.0")
	check(not Protocol.parse_envelope('{"protocol_version":2,"request_id":"r","type":"t","payload":{}}')["ok"], "integer major 2 rejected")
	check(not Protocol.parse_envelope('{"protocol_version":"1.0","request_id":"","type":"t","payload":{}}')["ok"], "empty request_id rejected")


func test_commands() -> void:
	var set_order := Protocol.command_set_unit_order(3, Protocol.order_wire("Fortify"))
	check(set_order == {"command": {"type": "set_unit_order", "data": {"unit_id": 3, "order": {"type": "fortify"}}}}, "SetUnitOrder payload shape (serde adjacent tagging)")
	var move := Protocol.command_set_unit_order(3, Protocol.order_move_to(99))
	check(move["command"]["data"]["order"] == {"type": "move_to", "data": {"target": 99}}, "MoveTo order is adjacently tagged")
	check(Protocol.command_skip_unit(5) == {"command": {"type": "skip_unit", "data": {"unit_id": 5}}}, "SkipUnit payload shape")
	check(Protocol.command_respond_to_event(7, "ration")["command"]["data"] == {"event_id": 7, "choice_id": "ration"}, "RespondToEvent payload shape")
	var framed := Protocol.parse_envelope(Protocol.build_envelope("submit_command", Protocol.command_skip_unit(5), "c-9"))
	check(framed["ok"] and framed["envelope"]["payload"]["command"]["type"] == "skip_unit", "command survives the envelope round trip")


func test_hex_index() -> void:
	check(Hex.cell_of_index(0, 24) == Vector2i(0, 0) and Hex.cell_of_index(25, 24) == Vector2i(1, 1), "tile index to cell is row-major")
	for cell in [Vector2i(0, 0), Vector2i(23, 5), Vector2i(7, 15)]:
		check(Hex.cell_of_index(Hex.index_of_cell(cell, 24), 24) == cell, "index round-trips %s" % cell)
	var even := Hex.edge_offsets(2)
	check(even.size() == 6 and even[0] == Vector2i(0, -1), "edge offsets follow neighbor order (even row)")
	check(Hex.edge_offsets(3)[0] == Vector2i(1, -1), "edge offsets follow neighbor order (odd row)")


func test_server_view_adapter() -> void:
	var catalog := _load_catalog()
	var view := _adapt(_load_view_payload(), catalog)
	check(view.live and view.turn == 42 and view.civ_id == 0 and view.world_id == 20261003, "header fields come from the view")
	check(view.map_width == 24 and view.map_height >= 13 and view.tiles.size() == view.map_height, "map dimensions: width given, height inferred from the highest tile")
	for row in view.tiles:
		check(row.size() == 24, "every row has map_width tiles")
	check(view.tile_at(Vector2i(10, 7))["fog"] == "visible" and view.tile_at(Vector2i(10, 7))["biome"] == "planicie", "own capital tile: visible plains (terrain 0)")
	check(view.tile_at(Vector2i(0, 0))["fog"] == "unknown" and not view.tile_at(Vector2i(0, 0)).has("biome"), "tiles the server did not send stay unknown and blank")
	var remembered := 0
	var visible := 0
	for row in view.tiles:
		for tile in row:
			if tile["fog"] == "remembered":
				remembered += 1
			elif tile["fog"] == "visible":
				visible += 1
	check(visible > 0 and remembered > 0, "visible and remembered tiles are kept apart")
	check(view.tile_at(Vector2i(10, 7)).has("city") and view.tile_at(Vector2i(10, 7))["city"]["own"], "own city is on the capital tile")
	check(view.tile_at(Vector2i(15, 6)).has("city") and not view.tile_at(Vector2i(15, 6))["city"]["own"], "foreign city is marked as not own")
	check(view.capital == Vector2i(10, 7), "capital = first own city")
	var borders := 0
	for row in view.tiles:
		for tile in row:
			borders += tile.get("borders", []).size()
	check(borders > 0, "ownership produces border edges")
	check(view.tile_at(Vector2i(10, 7)).get("borders", []).is_empty(), "interior tile of own territory has no border edge")

	check(view.units.size() == 7, "all seven units are adapted")
	var scout: Dictionary = view.unit_by_id(3)
	check(scout["name"] == "Batedor" and scout["role"] == "exploration" and scout["cell"] == Vector2i(12, 6), "unit type, role and cell")
	check(scout["own"] and scout["order"] == "Idle" and scout["movement_max"] == 3 and scout["movement_left"] == 3, "own unit: order and movement from view + catalog")
	var worker: Dictionary = view.unit_by_id(6)
	check(worker["order"] == "MoveTo" and worker["order_target"] == Vector2i(11, 9), "MoveTo order decodes its target tile index to a cell")
	check(view.unit_by_id(4)["order"] == "Fortify" and view.unit_by_id(7)["order"] == "Explore", "string orders decode")
	check(view.unit_by_id(8)["skipped_turn"] == 42, "skipped_turn is kept")
	check(not view.unit_by_id(20)["own"] and view.unit_by_id(20)["owner"] == 1, "foreign unit is not own")
	check(view.units_at(Vector2i(10, 7)).size() == 1 and view.units_at(Vector2i(10, 7))[0]["id"] == 4, "units_at finds the militia in the capital")
	check(view.idle_units == [3, 5], "idle_units come from the server list")
	check(view.civ_colors[0] != view.civ_colors[1], "each civilization gets its own color")

	check(view.research["current"] == "tech.storage" and view.research["done"].has("tech.council") and view.research["progress"]["tech.storage"] == 6, "research state of the own civilization")
	check(view.header_chips.size() == 4 and view.header_chips[0]["value"] == 18, "header chips: wealth, knowledge, culture, cohesion")
	check(view.agenda.size() == 1 and view.agenda[0]["event_id"] == 7 and view.agenda[0]["options"].size() == 2, "pending event becomes an agenda card with its choices")
	check(view.agenda[0]["deadline_turns"] == 3 and view.agenda[0]["options"][1]["sacrifice"] == "wealth -3", "event deadline and choice effects")
	check(view.pending_events.size() == 1, "pending events are kept for the response command")


func test_adapter_tolerates_missing_fields() -> void:
	var payload := _load_view_payload()
	payload.erase("idle_units")
	payload.erase("pending_events")
	payload.erase("cities")
	for entry in payload["units"]:
		entry["unit"].erase("order")
		entry["unit"].erase("skipped_turn")
	var view := _adapt(payload)  # no catalog either
	check(view.idle_units.is_empty(), "no idle_units from an older server: queue stays empty")
	check(not view.ready_blocked(), "no gate without server idle list")
	check(view.unit_by_id(3)["order"] == "" and view.unit_by_id(3)["skipped_turn"] == -1, "missing order/skipped_turn are tolerated")
	check(view.unit_by_id(3)["name"] == "Scout", "name falls back to the humanized type without a catalog")
	check(view.unit_by_id(3)["role"] == "exploration", "role falls back to the built-in table without a catalog")
	check(view.capital == Vector2i(12, 6), "capital falls back to the first own unit when there is no city")
	check(view.agenda.is_empty(), "no pending events, no cards")
	payload["map_height"] = 20
	check(_adapt(payload).map_height == 20, "optional map_height is honored")
	payload["idle_units"] = [3, 20, 999]
	check(_adapt(payload).idle_units == [3], "idle list keeps only own, visible units")
	var strange := ServerView.parse_order({"Weird": 1}, 24)
	check(strange["kind"] == "", "unrecognized order decodes to unknown, not a crash")


func test_adapter_rejects_bad_input() -> void:
	var payload := _load_view_payload()
	payload.erase("map_width")
	check(not ServerView.from_payload(payload)["ok"], "missing map_width rejected")
	payload = _load_view_payload()
	payload["tiles"][0]["terrain"] = 42
	check(not ServerView.from_payload(payload)["ok"], "unknown terrain id rejected")
	payload = _load_view_payload()
	payload["map_width"] = 0
	check(not ServerView.from_payload(payload)["ok"], "zero width rejected")
	payload = _load_view_payload()
	payload.erase("tiles")
	check(not ServerView.from_payload(payload)["ok"], "missing tiles rejected")


func test_idle_queue_and_gate() -> void:
	var view := _adapt(_load_view_payload(), _load_catalog())
	check(view.awaiting_count() == 2 and view.ready_blocked(), "two idle units block Pronto")
	check(WorldView.gate_text(2) == "2 unidades aguardam ordem" and WorldView.gate_text(1) == "1 unidade aguarda ordem", "gate copy, plural and singular")
	view.apply_local_order(3, "Fortify")
	check(view.idle_units == [5] and view.unit_by_id(3)["order"] == "Fortify", "ordering a unit removes it from the queue")
	check(view.awaiting_count() == 1 and view.ready_blocked(), "one idle unit still blocks")
	view.apply_local_skip(5)
	check(view.idle_units.is_empty() and not view.ready_blocked(), "skipping the last idle unit opens the gate")
	check(view.unit_by_id(5)["skipped_turn"] == view.turn, "skip records the current turn")
	view.apply_local_order(3, "Idle")
	check(view.idle_units == [3], "an idle order with movement left re-enters the queue")
	view.apply_local_order(7, "Idle")
	check(view.idle_units == [3], "a unit without movement left never enters the queue")
	view.apply_local_order(6, "MoveTo", Vector2i(2, 2))
	check(view.unit_by_id(6)["order_target"] == Vector2i(2, 2) and not view.idle_units.has(6), "MoveTo stores its target")
	view.apply_local_order(404, "Fortify")
	check(view.idle_units == [3], "orders for unknown units are ignored")


func test_agenda_ready_gate() -> void:
	var agenda := AgendaView.new()
	agenda.build([], 42)
	var focused: Array = []
	agenda.unit_focus_requested.connect(func(id: int) -> void: focused.append(id))
	check(not agenda._ready_button.disabled and agenda._ready_button.text == "Pronto", "Pronto starts enabled")
	agenda.set_idle_units([{"id": 3, "label": "Batedor · (12, 6)"}, {"id": 5, "label": "Colono · (9, 7)"}])
	check(agenda._ready_button.disabled and agenda._ready_button.text == "2 unidades aguardam ordem", "Pronto disabled with the unit count")
	check(agenda._idle_box.get_child_count() == 3, "queue shows a heading and one row per idle unit")
	var first_row: Button = agenda._idle_box.get_child(1)
	check(first_row.custom_minimum_size.y >= 44, "queue rows are at least 44 px tall")
	first_row.pressed.emit()
	check(focused == [3], "tapping a queue row asks the map to focus that unit")
	agenda.set_idle_units([{"id": 5, "label": "Colono · (9, 7)"}])
	check(agenda._ready_button.text == "1 unidade aguarda ordem", "singular copy in the button")
	agenda.set_idle_units([])
	check(not agenda._ready_button.disabled and agenda._ready_button.text == "Pronto", "gate opens when the queue is empty")
	agenda.mark_ready(true)
	check(agenda._ready_button.disabled and agenda._ready_button.text == "Pronto (enviado)", "sent state locks the button")
	agenda.reset_ready()
	check(not agenda._ready_button.disabled, "reset_ready reopens the button")
	agenda.free()


func test_unit_panel_actions() -> void:
	var view := _adapt(_load_view_payload(), _load_catalog())
	check(UnitPanel.legal_actions(view.unit_by_id(3)) == ["move", "explore", "fortify", "skip"], "scout: move, explore, fortify, skip")
	check(UnitPanel.legal_actions(view.unit_by_id(4)) == ["move", "fortify", "skip"], "militia: no explore")
	check(UnitPanel.legal_actions(view.unit_by_id(5)) == ["move", "found", "skip"], "settler: move, found a city, skip")
	check(UnitPanel.legal_actions(view.unit_by_id(20)).is_empty(), "foreign unit offers no orders")
	check(UnitPanel.order_label(view.unit_by_id(3)) == "aguardando ordem", "idle label")
	check(UnitPanel.order_label(view.unit_by_id(6)) == "indo para (11, 9)", "MoveTo label shows the destination")
	check(UnitPanel.order_label({"order": ""}) == "ordem desconhecida", "unknown order label")


func test_catalog_layout() -> void:
	var catalog := _load_catalog()
	check(catalog.techs.size() == 17 and catalog.unit_types.size() == 10 and catalog.version == 1, "catalog counts and version")
	check(catalog.depth("tech.foraging") == 0 and catalog.depth("tech.storage") == 1 and catalog.depth("tech.irrigation") == 2 and catalog.depth("tech.crop_rotation") == 3, "depth follows the prerequisite chain")
	check(catalog.depth("tech.granary_administration") == 2, "depth is the longest prerequisite path (storage 1, recordkeeping 1)")
	check(catalog.depth("tech.market_charter") == 3, "market charter sits three levels deep")
	var columns: Array = catalog.columns()
	check(columns.size() == 4, "four columns for a depth-3 tree")
	var placed := {}
	var column_of := {}
	for depth in columns.size():
		for id in columns[depth]:
			check(not placed.has(id), "tech %s is in exactly one column" % id)
			placed[id] = true
			column_of[id] = depth
	check(placed.size() == catalog.techs.size(), "every tech is placed")
	for id in catalog.techs:
		for prereq in catalog.techs[id]["prerequisites"]:
			check(column_of[prereq] < column_of[id], "%s sits right of its prerequisite %s" % [id, prereq])
	var research := {"current": "tech.storage", "done": ["tech.foraging", "tech.council"], "progress": {"tech.storage": 6}}
	check(catalog.status("tech.foraging", research) == "done", "mastered tech")
	check(catalog.status("tech.storage", research) == "current", "current research")
	check(catalog.status("tech.irrigation", research) == "locked", "prerequisite not mastered: locked")
	check(catalog.status("tech.herbal_care", research) == "available", "all prerequisites mastered: available")
	check(catalog.status("tech.paths", research) == "available", "root tech is available")
	check(catalog.progress("tech.storage", research) == {"have": 6, "cost": 14}, "progress of the current research")
	check(catalog.progress("tech.foraging", research)["have"] == catalog.progress("tech.foraging", research)["cost"], "mastered tech reads full")
	check(catalog.progress("tech.irrigation", {"progress": {"tech.irrigation": 999}})["have"] == 20, "progress is capped at the cost")
	check(catalog.missing_prerequisites("tech.irrigation", research) == ["Armazenamento"], "locked tech names what it still needs")
	check(TechView.summary_text(catalog, research) == "2 de 17 dominadas. Em pesquisa: Armazenamento (6/14). Somente leitura.", "summary line")
	check(TechView.summary_text(catalog, {"current": "", "done": [], "progress": {}}).contains("Nenhuma pesquisa em andamento"), "summary without current research")


func test_catalog_rejects_bad_input() -> void:
	var cycle := CatalogView.from_payload({"technologies": [{"id": "a", "prerequisites": ["b"]}, {"id": "b", "prerequisites": ["a"]}]})
	check(not cycle["ok"], "prerequisite cycle rejected")
	var duplicate := CatalogView.from_payload({"technologies": [{"id": "a"}, {"id": "a"}]})
	check(not duplicate["ok"], "duplicate tech id rejected")
	check(not CatalogView.from_payload({"technologies": [{"name": "no id"}]})["ok"], "tech without id rejected")
	var tolerant := CatalogView.from_payload({"tech_tree": {"technologies": [{"id": "a"}, {"id": "b", "requires": ["a", "ghost"]}]}, "units": [{"id": "unit.x", "name": "X"}]})
	check(tolerant["ok"] and tolerant["catalog"].depth("b") == 1 and tolerant["catalog"].unit_types.has("unit.x"), "aliases accepted; unknown prerequisites ignored")


func test_session_store() -> void:
	var path := "user://test_sessions.cfg"
	DirAccess.remove_absolute(ProjectSettings.globalize_path(path))
	check(SessionStore.load_token(7, 1, path) == "", "no token before saving")
	SessionStore.save_token(7, 1, "tok-abc", path)
	SessionStore.save_token(7, 2, "tok-def", path)
	check(SessionStore.load_token(7, 1, path) == "tok-abc" and SessionStore.load_token(7, 2, path) == "tok-def", "tokens are kept per world and civilization")
	SessionStore.forget_token(7, 1, path)
	check(SessionStore.load_token(7, 1, path) == "" and SessionStore.load_token(7, 2, path) == "tok-def", "forgetting one token keeps the others")
	var fallback := SessionStore.load_last("ws://default/ws", path)
	check(fallback["url"] == "ws://default/ws" and fallback["world_id"] == 1 and fallback["civ"] == 0, "defaults before any session")
	SessionStore.save_last("ws://h:1/ws", 7, 2, path)
	var last := SessionStore.load_last("ws://default/ws", path)
	check(last["url"] == "ws://h:1/ws" and last["world_id"] == 7 and last["civ"] == 2, "last session is remembered")
	DirAccess.remove_absolute(ProjectSettings.globalize_path(path))


func test_views_build() -> void:
	var catalog := _load_catalog()
	var view := _adapt(_load_view_payload(), catalog)
	var tech := TechView.new()
	tech.build(catalog, view.research)
	check(tech.get_child_count() > 0, "tech view builds without errors")
	tech.free()
	var connect := ConnectView.new()
	var created: Array = []
	connect.create_requested.connect(func(url: String, seed: int, civs: int) -> void: created.append([url, seed, civs]))
	connect.build("ws://127.0.0.1:8100/ws", 5, 1)
	connect._on_create()
	check(created == [["ws://127.0.0.1:8100/ws", 1, 4]], "create emits url, seed and civilization count")
	var joined: Array = []
	connect.join_requested.connect(func(_url: String, world_id: int, civ: int) -> void: joined.append([world_id, civ]))
	connect._on_join()
	check(joined == [[5, 1]], "join emits the prefilled world and civilization")
	connect.set_join_target(9, 3)
	connect._on_join()
	check(joined[1] == [9, 3], "join target can be updated after a world is created")
	for button in connect._buttons:
		check(button.custom_minimum_size.y >= 44, "start screen buttons are at least 44 px tall")
	connect.free()


func test_found_and_produce_model() -> void:
	var catalog := _load_catalog()
	var payload := _load_view_payload()
	var view := _adapt(payload, catalog)
	check(view.home_cell == Vector2i(-1, -1) and not view.needs_capital(), "no home tile while a city exists")
	check(view.cities.size() == 2 and view.own_cities().size() == 1, "own and foreign cities are listed")
	var capital: Dictionary = view.city_by_id(0)
	check(capital["name"] == "Capital" and capital["cell"] == Vector2i(10, 7) and capital["own"], "own first city is the Capital")
	check(capital["population"] == 3 and capital["housing"] == 4 and capital["focus"] == "supply" and capital["stability"] == 62, "city summary fields")
	check(capital["queue"] == ["unit.worker"] and capital["unit_production"] == 4 and capital["yields"]["production"] == 2, "queue, progress and yields")
	check(view.city_by_id(1)["name"] == "Capital da civilização 2" and not view.city_by_id(1)["own"], "foreign city is named after its civilization")
	check(view.city_at(Vector2i(10, 7))["id"] == 0 and view.city_at(Vector2i(0, 0)).is_empty(), "city_at")

	# No city yet: the server names the starting tile.
	payload.erase("cities")
	payload["home_tile"] = 178
	view = _adapt(payload, catalog)
	check(view.home_cell == Vector2i(10, 7) and view.needs_capital(), "home_tile with no city asks for a capital")
	check(view.capital == Vector2i(10, 7), "the camera starts on the starting tile")
	payload["home_tile"] = null
	check(not _adapt(payload, catalog).needs_capital(), "null home_tile never asks for a capital")
	payload["home_tile"] = 178
	view = _adapt(payload, catalog)
	check(not view.is_founding(Vector2i(10, 7)), "nothing is being founded yet")
	view.apply_local_found(Vector2i(10, 7))
	check(view.is_founding(Vector2i(10, 7)), "accepted founding is remembered for the open turn")
	# A settler on the tile stops waiting for an order once its founding is accepted.
	view = _adapt(_load_view_payload(), catalog)
	check(view.idle_units == [3, 5], "settler 5 starts idle")
	view.apply_local_found(Vector2i(9, 7))
	check(view.idle_units == [3] and view.unit_by_id(5)["founding"], "founding with a settler removes it from the queue")
	check(UnitPanel.order_label(view.unit_by_id(5)) == "fundando uma cidade", "settler label while founding")
	check(UnitPanel.legal_actions(view.unit_by_id(5), view.tile_at(Vector2i(9, 7))) == ["move", "skip"], "no second founding for the same settler")
	view.apply_local_queue(0, "unit.scout")
	check(view.city_by_id(0)["queue"] == ["unit.worker", "unit.scout"], "accepted queue request is appended locally")
	view.apply_local_queue(404, "unit.scout")
	check(view.city_by_id(404).is_empty(), "queueing in an unknown city is ignored")


func test_found_and_queue_commands() -> void:
	check(Protocol.command_found_city(0, 178) == {"command": {"type": "found_city", "data": {"city_id": 0, "target": 178}}}, "FoundCity payload shape")
	check(Protocol.command_queue_unit(0, "unit.scout") == {"command": {"type": "queue_unit", "data": {"city_id": 0, "unit_type": "unit.scout"}}}, "QueueUnit payload shape")
	check(Protocol.settler_city_id(5) == 1000005, "settler city ids follow the bots' rule")
	var view := _adapt(_load_view_payload(), _load_catalog())
	var settler: Dictionary = view.unit_by_id(5)
	var plains: Dictionary = view.tile_at(settler["cell"])
	check(UnitPanel.legal_actions(settler, plains).has("found"), "settler on open land may found")
	check(not UnitPanel.legal_actions(settler, {"biome": "oceano", "fog": "visible"}).has("found"), "no founding at sea")
	check(not UnitPanel.legal_actions(settler, {"biome": "planicie", "city": {"id": 0}}).has("found"), "no founding on a city")
	check(not UnitPanel.legal_actions(view.unit_by_id(3), plains).has("found"), "only settlers found cities")


func test_capital_card() -> void:
	var agenda := AgendaView.new()
	root.add_child(agenda)
	agenda.build([], 0)
	var events: Array = []
	agenda.found_capital_requested.connect(func() -> void: events.append("found"))
	agenda.capital_focus_requested.connect(func(cell: Vector2i) -> void: events.append(cell))
	check(agenda._capital_box.get_child_count() == 0, "no capital card by default")
	agenda.set_capital_card(Vector2i(10, 7), false)
	check(agenda._capital_box.get_child_count() == 1, "capital card appears with a starting tile")
	var buttons: Array = agenda._capital_box.find_children("*", "Button", true, false)
	check(buttons.size() == 2, "card has the focus and found actions")
	for button in buttons:
		check(button.custom_minimum_size.y >= 44, "capital card buttons are at least 44 px tall")
	buttons[0].pressed.emit()
	buttons[1].pressed.emit()
	check(events == [Vector2i(10, 7), "found"], "the card centers on the starting tile and founds the capital")
	agenda.set_capital_card(Vector2i(10, 7), true)
	buttons = agenda._capital_box.find_children("*", "Button", true, false)
	check(buttons.size() == 1, "once ordered, the found action is gone (no repeated command)")
	agenda.set_capital_card(Vector2i(-1, -1), false)
	check(agenda._capital_box.get_child_count() == 0, "card disappears once the capital exists")
	agenda.queue_free()


func test_city_panel() -> void:
	var catalog := _load_catalog()
	var view := _adapt(_load_view_payload(), catalog)
	var options: Array = CityPanel.producible_units(catalog, view.research)
	var ids: Array = options.map(func(o: Dictionary) -> String: return o["id"])
	check(ids == ["unit.scout", "unit.militia", "unit.worker"], "only units whose technology is mastered, by name: %s" % [ids])
	check(options[0]["cost_text"] == "produção 12" and options[2]["cost_text"] == "produção 10, comida 4", "cost text from the catalog")
	var locked: Array = CityPanel.locked_units(catalog, view.research)
	check(locked.size() == 7 and locked.any(func(l: Dictionary) -> bool: return l["name"] == "Colono" and l["tech"] == "Armazenamento"), "locked units name the missing technology")
	check(CityPanel.producible_units(null, view.research).is_empty(), "no catalog, no options")
	var city: Dictionary = view.city_by_id(0)
	check(CityPanel.progress_text(city, catalog) == "Trabalhador: produção 4/10 (+2 por turno)", "queue head progress: %s" % CityPanel.progress_text(city, catalog))
	check(CityPanel.progress_text({"queue": []}, catalog) == "", "empty queue has no progress line")
	check(CityPanel.summary_text(city) == "População 3/4 · foco: Abastecimento · comida em estoque 2 · estabilidade 62", "city summary line")

	var panel := CityPanel.new()
	root.add_child(panel)
	var queued: Array = []
	var founded: Array = []
	panel.queue_requested.connect(func(unit_type: String) -> void: queued.append(unit_type))
	panel.found_capital_requested.connect(func() -> void: founded.append(true))
	panel.show_city(city, catalog, view.research)
	check(panel.visible and panel.city_id() == 0, "panel opens for the city")
	var buttons: Array = panel._box.find_children("*", "Button", true, false)
	check(buttons.size() == 3, "one production button per available unit")
	for button in buttons:
		check(button.custom_minimum_size.y >= 44, "production buttons are at least 44 px tall")
	buttons[0].pressed.emit()
	check(queued == ["unit.scout"], "tapping a unit asks to queue it")
	panel.show_founding(Vector2i(10, 7), false)
	buttons = panel._box.find_children("*", "Button", true, false)
	check(buttons.size() == 1 and buttons[0].text == "Fundar capital", "starting tile offers Fundar capital")
	buttons[0].pressed.emit()
	check(founded == [true], "Fundar capital is emitted")
	panel.show_founding(Vector2i(10, 7), true)
	check(panel._box.find_children("*", "Button", true, false).is_empty(), "no button once the founding was ordered")
	panel.clear()
	check(not panel.visible, "panel hides on clear")
	panel.queue_free()

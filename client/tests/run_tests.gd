extends SceneTree
## Headless tests: godot --headless --path client -s res://tests/run_tests.gd
## No network, no scene tree needed beyond this script.

const Hex := preload("res://scripts/hex.gd")
const Protocol := preload("res://scripts/protocol.gd")
const WorldView := preload("res://scripts/world_view.gd")

var _failures := 0
var _checks := 0


func _init() -> void:
	test_hex_neighbors()
	test_distance_and_wrap()
	test_center_and_pixel_to_cell()
	test_envelope()
	test_fixture()
	test_agenda_limits()
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

extends SceneTree
## Integration test of the live path: the real main scene, NetClient and Protocol against an
## in-process mock server (TCPServer + WebSocketPeer). The mock replays fixtures in the server's
## frame format. Not part of run_tests.gd because it opens a loopback socket.
##   godot --headless --path client -s res://tests/live_flow.gd

const PORT := 18123
const MAX_FRAMES := 1500

var _tcp := TCPServer.new()
var _peer: WebSocketPeer
var _failures := 0
var _checks := 0
var _received: Array = []  # decoded client frames
var _idle_remaining := [3, 5]
var _main: Node


func check(condition: bool, label: String) -> void:
	_checks += 1
	if not condition:
		_failures += 1
		printerr("FAIL: %s" % label)


func _initialize() -> void:
	_run()


func _run() -> void:
	_tcp.listen(PORT, "127.0.0.1")
	_main = load("res://scenes/main.tscn").instantiate()
	root.add_child(_main)
	await _frames(2)
	check(_main.mode == _main.Mode.CONNECT, "app starts on the connection screen")

	_main._on_create_requested("ws://127.0.0.1:%d/ws" % PORT, 5, 2)
	await _wait(func() -> bool: return _main.mode == _main.Mode.LIVE and _main.world != null and _main.world.live and _main._catalog != null)
	check(_main.world != null and _main.world.live, "create + join leads to the live view")
	check(_received.size() >= 2 and _received[0]["type"] == "create_world" and int(_received[0]["payload"]["seed"]) == 5 and int(_received[0]["payload"]["civs"]) == 2, "create_world sent with seed and civs")
	check(_received[1]["type"] == "join" and _received[1]["payload"]["world_id"] == 5 and _received[1]["payload"]["civ"] == 0, "join sent right after the world was created")
	check(_main._catalog != null and _main._catalog.techs.size() == 17, "catalog push is parsed")
	check(_main.world.unit_by_id(3)["name"] == "Batedor", "unit names use the catalog that arrived after the snapshot")
	check(_main._nav_buttons["pesquisa"].visible, "Pesquisa tab appears once the catalog is there")
	check(_main.world.idle_units == [3, 5], "idle units from the server")
	check(_main._agenda._ready_button.disabled and _main._agenda._ready_button.text == "2 unidades aguardam ordem", "Pronto is gated in the Pauta")

	_main._on_ready_pressed({})
	await _frames(10)
	check(not _received.any(func(f: Dictionary) -> bool: return f["type"] == "ready"), "Pronto is not sent while units wait")

	_main._on_unit_focus_requested(3)
	check(_main._selected_unit == 3 and _main._screens["mapa"].visible, "queue tap shows the map with the unit selected")
	_main._on_unit_action("fortify")
	await _wait(func() -> bool: return _main.world.idle_units == [5])
	var command: Dictionary = _received.back()
	check(command["type"] == "submit_command" and command["payload"]["command"]["type"] == "set_unit_order" and int(command["payload"]["command"]["data"]["unit_id"]) == 3 and command["payload"]["command"]["data"]["order"] == {"type": "fortify"}, "Fortificar sends SetUnitOrder")
	check(_main.world.unit_by_id(3)["order"] == "Fortify", "accepted order is shown locally")

	_main._select_unit(5)
	_main._on_unit_action("move")
	check(_main._move_mode and _main._map.target_mode, "Mover enters target mode")
	_main._on_target_chosen(Vector2i(9, 8))
	await _wait(func() -> bool: return _main.world.idle_units.is_empty())
	var move: Dictionary = _received.back()
	check(int(move["payload"]["command"]["data"]["order"]["data"]["target"]) == 8 * 24 + 9, "destination is sent as a tile index")
	check(not _main._move_mode and not _main._map.target_mode, "target mode ends after choosing")
	check(_main.world.unit_by_id(5)["order_target"] == Vector2i(9, 8), "MoveTo is reflected on the unit")

	_main._refresh_units()
	check(not _main._agenda._ready_button.disabled and _main._agenda._ready_button.text == "Pronto", "Pronto opens when every unit has an order")

	# A snapshot of the same turn that does not yet reflect the accepted orders must not reopen the gate.
	_main._on_envelope({"type": "state_snapshot", "request_id": "", "payload": _main._last_payload.duplicate(true)})
	check(_main.world.idle_units.is_empty(), "accepted orders survive a same-turn snapshot refresh")

	_main._on_ready_pressed({})
	await _wait(func() -> bool: return _main._agenda._ready_button.text == "Pronto (enviado)")
	check(_received.any(func(f: Dictionary) -> bool: return f["type"] == "ready"), "ready is sent once the gate is open")

	_main._on_error("", {"reason": "units_awaiting_orders", "detail": "units [3]"})
	await _frames(5)
	check(_received.any(func(f: Dictionary) -> bool: return f["type"] == "get_snapshot"), "units_awaiting_orders triggers a resync")

	# Found and produce. A fresh world: no city, the server names the starting tile.
	var empty_view: Dictionary = _view()
	empty_view.erase("cities")
	empty_view["home_tile"] = 178
	empty_view["idle_units"] = []
	_main._on_envelope({"type": "state_snapshot", "request_id": "", "payload": empty_view})
	check(_main.world.needs_capital() and _main.world.home_cell == Vector2i(10, 7), "a world without cities asks for a capital")
	var card_buttons: Array = _main._agenda._capital_box.find_children("*", "Button", true, false)
	check(card_buttons.size() == 2, "the Pauta shows the capital card with its two actions")
	card_buttons[0].pressed.emit()
	check(_main._screens["mapa"].visible and _main._founding_selected and _main._city_panel.visible, "the card centers the map on the starting tile and opens it")
	var sent_before := _received.size()
	card_buttons = _main._agenda._capital_box.find_children("*", "Button", true, false)
	card_buttons[1].pressed.emit()
	await _wait(func() -> bool: return _main.world.is_founding(Vector2i(10, 7)))
	var found: Dictionary = _received[sent_before]["payload"]["command"]
	check(found["type"] == "found_city" and int(found["data"]["city_id"]) == 0 and int(found["data"]["target"]) == 178, "Fundar capital sends FoundCity with the civilization id at the starting tile")
	check(_main._agenda._capital_box.find_children("*", "Button", true, false).size() == 1, "after acceptance the card no longer offers to found again")
	# The city exists in the next view: tapping it opens the production panel.
	_main._on_envelope({"type": "state_snapshot", "request_id": "", "payload": _view()})
	_main._founding_selected = false
	_main._clear_unit_selection()
	_main._on_cell_selected(Vector2i(10, 7))
	check(_main._selected_city == 0 and _main._city_panel.visible and _main._city_panel.city_id() == 0, "tapping the own city opens its panel")
	var production: Array = _main._city_panel._box.find_children("*", "Button", true, false)
	check(production.size() == 3, "the panel lists the units the civilization can produce")
	sent_before = _received.size()
	production[0].pressed.emit()
	await _wait(func() -> bool: return _main.world.city_by_id(0)["queue"].size() == 2)
	var queue: Dictionary = _received[sent_before]["payload"]["command"]
	check(queue["type"] == "queue_unit" and int(queue["data"]["city_id"]) == 0 and queue["data"]["unit_type"] == "unit.scout", "a production button sends QueueUnit")
	# A settler founds a city where it stands.
	_main._select_unit(5)
	check(_main._unit_panel._buttons["found"].visible, "the settler card offers Fundar cidade")
	sent_before = _received.size()
	_main._on_unit_action("found")
	await _wait(func() -> bool: return _main.world.is_founding(Vector2i(9, 7)))
	var settle: Dictionary = _received[sent_before]["payload"]["command"]
	check(settle["type"] == "found_city" and int(settle["data"]["city_id"]) == 1000005 and int(settle["data"]["target"]) == 7 * 24 + 9, "Fundar cidade sends FoundCity with the settler-derived id")
	check(not _main.world.idle_units.has(5) and not _main._unit_panel._buttons["found"].visible, "the settler stops waiting and cannot found twice")

	var stored: String = _main.SessionStore.load_token(5, 0)
	check(stored == "tok-5-0", "session token is stored for reconnection")
	_main.SessionStore.forget_token(5, 0)

	print("%d live checks, %d failures" % [_checks, _failures])
	if _failures == 0:
		print("LIVE FLOW PASSED")
	quit(0 if _failures == 0 else 1)


func _frames(count: int) -> void:
	for i in count:
		await process_frame
		_serve()


func _wait(condition: Callable) -> void:
	for i in MAX_FRAMES:
		await process_frame
		_serve()
		if condition.call():
			return
	check(false, "timed out waiting for a condition")


func _serve() -> void:
	if _peer == null and _tcp.is_connection_available():
		_peer = WebSocketPeer.new()
		_peer.accept_stream(_tcp.take_connection())
	if _peer == null:
		return
	_peer.poll()
	while _peer.get_ready_state() == WebSocketPeer.STATE_OPEN and _peer.get_available_packet_count() > 0:
		var frame: Dictionary = JSON.parse_string(_peer.get_packet().get_string_from_utf8())
		_received.append(frame)
		_respond(frame)


func _send(kind: String, request_id: Variant, payload: Dictionary) -> void:
	_peer.send_text(JSON.stringify({"protocol_version": "1.0", "request_id": request_id, "type": kind, "payload": payload}))


func _view() -> Dictionary:
	return JSON.parse_string(FileAccess.get_file_as_string("res://fixtures/server_view.json"))


func _respond(frame: Dictionary) -> void:
	var rid: String = frame["request_id"]
	var payload: Dictionary = frame["payload"]
	match frame["type"]:
		"create_world":
			_send("world_created", rid, {"world_id": payload["seed"], "civs": [0, 1], "turn": 42, "state_hash": "00"})
		"join":
			_send("joined", rid, {"world_id": payload["world_id"], "civ": payload["civ"], "session_token": "tok-%d-%d" % [payload["world_id"], payload["civ"]], "turn": 42})
			_send("state_snapshot", rid, _view())
			_send("ready_state", rid, {"turn": 42, "present": [0], "ready": []})
			_send("catalog", null, JSON.parse_string(FileAccess.get_file_as_string("res://fixtures/server_catalog.json")))
		"submit_command":
			_send("command_accepted", rid, {"command_id": 1, "accepted_sequence": 1, "turn": 42})
		"ready":
			_send("ready_state", rid, {"turn": 42, "present": [0], "ready": [0]})
		"get_snapshot":
			_send("state_snapshot", rid, _view())

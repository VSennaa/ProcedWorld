extends SceneTree
## Integration test of the live path: the real main scene, NetClient and Protocol against an
## in-process mock server (TCPServer + WebSocketPeer). The mock replays fixtures in the server's
## frame format. Not part of run_tests.gd because it opens a loopback socket.
##   godot --headless --path client -s res://tests/live_flow.gd

const SessionStore := preload("res://scripts/session_store.gd")

const PORT := 18124
const MAX_FRAMES := 1500
const SESSION_PATH := "res://tests/.tmp_live_sessions.cfg"

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
	DirAccess.remove_absolute(ProjectSettings.globalize_path(SESSION_PATH))
	check(_tcp.listen(PORT, "127.0.0.1") == OK, "mock server listens on loopback")
	_main = load("res://scenes/main.tscn").instantiate()
	root.add_child(_main)
	_main._session_store_path = SESSION_PATH
	await _frames(2)
	check(_main.mode == _main.Mode.CONNECT, "app starts on the connection screen")

	_main._on_create_requested("ws://127.0.0.1:%d/ws" % PORT, 5, 2)
	await _wait(func() -> bool: return _main.mode == _main.Mode.LIVE and _main.world != null and _main.world.live and _main._catalog != null)
	check(_main.world != null and _main.world.live, "create + join leads to the live view")
	check(_received.size() >= 2 and _received[0]["type"] == "create_world" and int(_received[0]["payload"]["seed"]) == 5 and int(_received[0]["payload"]["civs"]) == 2, "create_world sent with seed and civs")
	check(_received[1]["type"] == "join" and _received[1]["payload"]["world_id"] == 5 and _received[1]["payload"]["civ"] == 0, "join sent right after the world was created")
	check(_main._catalog != null and _main._catalog.techs.size() == 17, "catalog push is parsed")
	check(SessionStore.load_token(5, 0, SESSION_PATH) == "tok-5-0", "joined token is saved through the live flow")
	check(_main.world.unit_by_id(3)["name"] == "Batedor", "unit names use the catalog that arrived after the snapshot")
	check(_main._nav_buttons["pesquisa"].visible, "Pesquisa tab appears once the catalog is there")
	_main.show_screen("pesquisa")
	var research_buttons: Array = _main._tech.find_children("*", "Button", true, false)
	var research_before := _received.size()
	research_buttons[0].pressed.emit()
	await _wait(func() -> bool: return _main.world.research["current"] == "tech.paths")
	var research: Dictionary = _received[research_before]
	check(research["type"] == "submit_command" and research["payload"]["command"] == {"type": "set_research", "data": {"research": "tech.paths"}}, "available technology sends SetResearch with its id")
	check(_main._tech.get_child_count() > 0, "research tree rebuilds after accepted selection")
	_main.show_screen("pauta")
	check(_main.world.idle_units == [3, 5], "idle units from the server")
	check(_main._agenda._ready_button.disabled and _main._agenda._ready_button.text == "2 unidades aguardam ordem", "Pronto is gated in the Pauta")

	_main._on_ready_pressed({})
	await _frames(10)
	check(not _received.any(func(f: Dictionary) -> bool: return f["type"] == "ready"), "Pronto is not sent while units wait")

	_main._agenda._on_option_pressed("event-7", "ration")
	check(_main._decisions() == [{"kind": "unit", "id": 3}, {"kind": "unit", "id": 5}], "two idle units remain as decisions")
	_main.show_screen("mapa")
	_main._fab.pressed.emit()
	check(_main._selected_unit == 3 and _main._screens["mapa"].visible, "the compact button centers and selects the first idle unit")
	_main._clear_unit_selection()
	_main._on_unit_focus_requested(3)
	check(_main._selected_unit == 3 and _main._screens["mapa"].visible, "queue tap shows the map with the unit selected")
	check(_main._unit_panel._buttons["attack"].visible, "an adjacent visible foreign unit enables Atacar")
	var attack_before := _received.size()
	_main._on_unit_action("attack:20")
	await _wait(func() -> bool: return _received.size() > attack_before)
	var attack: Dictionary = _received[attack_before]["payload"]["command"]
	check(attack["type"] == "declare_attack" and int(attack["data"]["attacker"]) == 3 and int(attack["data"]["target"]) == 20, "Atacar sends DeclareAttack with attacker and target")
	await _wait(func() -> bool: return int(_main.world.unit_by_id(3).get("attacked_turn", -1)) == _main.world.turn)
	attack_before = _received.size()
	_main._on_unit_action("attack:20")
	await _frames(8)
	check(_received.size() == attack_before, "Atacar is not sent after an attack was accepted")
	var fortify_before := _received.size()
	_main._on_unit_action("fortify")
	await _wait(func() -> bool: return _received.size() > fortify_before)
	var command: Dictionary = _received[fortify_before]
	check(command["type"] == "submit_command" and command["payload"]["command"]["type"] == "set_unit_order" and int(command["payload"]["command"]["data"]["unit_id"]) == 3 and command["payload"]["command"]["data"]["order"] == {"type": "fortify"}, "Fortificar sends SetUnitOrder")
	await _wait(func() -> bool: return _main.world.unit_by_id(3)["order"] == "Fortify")
	check(_main.world.unit_by_id(3)["order"] == "Fortify", "accepted order is shown locally")
	attack_before = _received.size()
	_main._on_unit_action("attack:20")
	await _frames(8)
	check(_received.size() == attack_before, "Atacar is not sent after an order was accepted")
	var sentry_from := _received.size()
	_main._on_unit_action("sentry")
	await _wait(func() -> bool: return _main.world.unit_by_id(3)["order"] == "Sentry")
	var sentry: Dictionary = _received[sentry_from]
	check(sentry["payload"]["command"]["type"] == "set_unit_order" and sentry["payload"]["command"]["data"]["order"] == {"type": "sentry"}, "Prontidão sends SetUnitOrder sentry")
	check(_main.world.idle_units == [5], "a unit in prontidão stays out of the queue")

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

	# Compact Pronto button: hidden on the Pauta, counting decisions elsewhere, walking them in order.
	_main.show_screen("pauta")
	check(not _main._fab.visible, "the compact button is hidden on the Pauta")
	_main.show_screen("mapa")
	check(_main._fab.visible and _main._fab.count == 1 and _main._fab._badge_label.text == "1", "the compact button counts the unanswered event")
	_main._fab.pressed.emit()
	check(_main._screens["evento"].visible, "with an event left, the button opens its accessible detail")
	var event_buttons: Array = _main._event.find_children("*", "Button", true, false)
	event_buttons[1].pressed.emit()
	check(_main._screens["pauta"].visible and _main._agenda.choices["event-7"] == "ration", "event detail records the response in the Pauta draft")
	check(_main._fab.count == 0, "answering the event clears the badge")
	_main.show_screen("mapa")
	check(_main._fab.visible and not _main._fab._badge.visible, "no decisions: ready icon, no badge")
	var ready_before := _received.filter(func(f: Dictionary) -> bool: return f["type"] == "ready").size()
	_main._fab.pressed.emit()
	await _wait(func() -> bool: return _main._fab.waiting)
	check(_received.filter(func(f: Dictionary) -> bool: return f["type"] == "ready").size() == ready_before + 1, "the compact button sends ready through the Pauta path")
	check(_received.any(func(f: Dictionary) -> bool: return f["type"] == "submit_command" and f["payload"]["command"]["type"] == "respond_to_event"), "the chosen event response is sent with ready")
	await _wait(func() -> bool: return _main._agenda._ready_button.text == "Pronto (enviado)")
	check(_main._fab.disabled, "after Pronto the compact button waits")
	_main._fab.pressed.emit()
	await _frames(5)
	check(_received.filter(func(f: Dictionary) -> bool: return f["type"] == "ready").size() == ready_before + 1, "a waiting button does not send ready twice")

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
	var production: Array = _main._city_panel._box.find_children("*", "Button", true, false).filter(func(b: Button) -> bool: return b.text.begins_with("Produzir"))
	check(production.size() == 3, "the panel lists the units the civilization can produce")
	sent_before = _received.size()
	production[0].pressed.emit()
	await _wait(func() -> bool: return _main.world.city_by_id(0)["queue"].size() == 2)
	var queue: Dictionary = _received[sent_before]["payload"]["command"]
	check(queue["type"] == "queue_unit" and int(queue["data"]["city_id"]) == 0 and queue["data"]["unit_type"] == "unit.scout", "a production button sends QueueUnit")
	# Queue management: move the new Scout up, then remove it; the server sees both commands.
	var icons: Array = _main._city_panel._box.find_children("*", "Button", true, false).filter(func(b: Button) -> bool: return b.text in ["▲", "▼", "✕"])
	sent_before = _received.size()
	icons[3].pressed.emit()  # second row, up
	await _wait(func() -> bool: return _main.world.city_by_id(0)["queue"][0] == "unit.scout")
	var queue_move: Dictionary = _received[sent_before]["payload"]["command"]
	check(queue_move["type"] == "move_queued_unit" and int(queue_move["data"]["city_id"]) == 0 and int(queue_move["data"]["from"]) == 1 and int(queue_move["data"]["to"]) == 0, "the up arrow sends MoveQueuedUnit")
	icons = _main._city_panel._box.find_children("*", "Button", true, false).filter(func(b: Button) -> bool: return b.text in ["▲", "▼", "✕"])
	sent_before = _received.size()
	icons[2].pressed.emit()  # first row, remove
	await _wait(func() -> bool: return _main.world.city_by_id(0)["queue"].size() == 1)
	var drop: Dictionary = _received[sent_before]["payload"]["command"]
	check(drop["type"] == "remove_queued_unit" and int(drop["data"]["city_id"]) == 0 and int(drop["data"]["index"]) == 0, "the remove button sends RemoveQueuedUnit")
	# Focus selector.
	var focus_buttons: Array = _main._city_panel._box.find_children("*", "Button", true, false).filter(func(b: Button) -> bool: return b.text == "Construção")
	sent_before = _received.size()
	focus_buttons[0].pressed.emit()
	await _wait(func() -> bool: return _main.world.city_by_id(0)["focus"] == "build")
	var focus: Dictionary = _received[sent_before]["payload"]["command"]
	check(focus["type"] == "set_city_focus" and focus["data"]["focus"] == "build", "the focus selector sends SetCityFocus")
	# A worker receives exactly the construction choices authorized by the server.
	_main._select_unit(8)
	var build_buttons: Array = _main._unit_panel._choices.find_children("*", "Button", true, false)
	check(build_buttons.any(func(button: Button) -> bool: return button.text == "Construir: Plantação"), "worker panel shows the server-authorized improvement")
	sent_before = _received.size()
	_main._on_unit_action("build:improvement.farm")
	await _wait(func() -> bool: return _main.world.unit_by_id(8)["order"] == "Build")
	var build: Dictionary = _received[sent_before]["payload"]["command"]
	check(build["type"] == "set_unit_order" and build["data"]["unit_id"] == 8 and build["data"]["order"] == {"type": "build", "data": {"improvement": "improvement.farm"}}, "Construir sends the Build order")
	# A settler founds a city where it stands.
	_main._select_unit(5)
	check(_main._unit_panel._buttons["found"].visible, "the settler card offers Fundar cidade")
	sent_before = _received.size()
	_main._on_unit_action("found")
	await _wait(func() -> bool: return _main.world.is_founding(Vector2i(9, 7)))
	var settle: Dictionary = _received[sent_before]["payload"]["command"]
	check(settle["type"] == "found_city" and int(settle["data"]["city_id"]) == 1000005 and int(settle["data"]["target"]) == 7 * 24 + 9, "Fundar cidade sends FoundCity with the settler-derived id")
	check(not _main.world.idle_units.has(5) and not _main._unit_panel._buttons["found"].visible, "the settler stops waiting and cannot found twice")

	check(_main._session == {"world_id": 5, "civ": 0}, "joined payload establishes the live session")
	# Re-enter through a fresh scene: it must load and send the token saved by the first join.
	_main._on_leave_pressed()
	await _frames(4)
	_main.queue_free()
	_peer = null
	_main = load("res://scenes/main.tscn").instantiate()
	root.add_child(_main)
	_main._session_store_path = SESSION_PATH
	await _frames(2)
	var reentry_before := _received.size()
	_main._on_join_requested("ws://127.0.0.1:%d/ws" % PORT, 5, 0)
	await _wait(func() -> bool: return _main.mode == _main.Mode.LIVE and _main.world != null)
	var reentry: Dictionary = _received[reentry_before]
	check(reentry["type"] == "join" and reentry["payload"].get("session_token", "") == "tok-5-0", "fresh client reentry sends the saved token")
	DirAccess.remove_absolute(ProjectSettings.globalize_path(SESSION_PATH))

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

extends SceneTree
## End-to-end check against a real pw-server (not run in CI: needs a server).
##   PW_E2E_URL=ws://127.0.0.1:18100/ws godot --headless --path client -s res://tests/real_server.gd
## Creates a world, founds the capital from the Pauta card, queues a Scout, plays turns
## (ordering idle units) until the Scout is produced, orders it and plays one more turn.

const MAX_FRAMES := 3000
const MAX_TURNS := 60

var _failures := 0
var _checks := 0
var _main: Node


func check(condition: bool, label: String) -> void:
	_checks += 1
	print(("ok   " if condition else "FAIL ") + label)
	if not condition:
		_failures += 1


func _initialize() -> void:
	_run()


func _run() -> void:
	var url := OS.get_environment("PW_E2E_URL")
	if url.is_empty():
		printerr("set PW_E2E_URL")
		quit(2)
		return
	_main = load("res://scenes/main.tscn").instantiate()
	root.add_child(_main)
	await _frames(2)
	_main._on_create_requested(url, 20261003, 4)
	var live := await _wait(func() -> bool: return _main.mode == _main.Mode.LIVE and _main.world != null and _main.world.live and _main._catalog != null)
	check(live, "create + join reach the live view with a catalog")
	if not live:
		_finish()
		return
	check(_main._catalog.techs.size() == 17, "catalog has the 17 technologies")
	check(_main._catalog.unit_types["unit.scout"].has("cost"), "catalog carries unit costs")
	var world = _main.world
	check(world.needs_capital() and world.home_cell.x >= 0, "a new world starts without a city and names the starting tile")
	check(not _main._agenda._capital_box.find_children("*", "Button", true, false).is_empty(), "the Pauta offers Fundar a capital")

	# Turn 0: found the capital from the Pauta card, then play the turn.
	_main._on_found_capital_requested()
	var ordered := await _wait(func() -> bool: return _main.world.is_founding(_main.world.home_cell))
	check(ordered, "the server accepts FoundCity for the starting tile")
	var turn: int = _main.world.turn
	_main._on_ready_pressed({})
	check(await _wait(func() -> bool: return _main.world.turn > turn), "Pronto advances the turn after founding (now %d)" % _main.world.turn)
	check(_main.world.own_cities().size() == 1 and not _main.world.needs_capital(), "the capital exists and the card is gone")
	var capital: Dictionary = _main.world.own_cities()[0]
	print("capital: %s" % [capital])

	# Queue a Scout in the capital through the city panel path.
	_main._on_cell_selected(capital["cell"])
	check(_main._selected_city == capital["id"] and _main._city_panel.visible, "tapping the capital opens the city panel")
	_main._on_queue_requested("unit.scout")
	check(await _wait(func() -> bool: return _main.world.city_by_id(capital["id"])["queue"] == ["unit.scout"]), "the server accepts QueueUnit and the panel shows the queue")

	# Advance turns, ordering idle units, until the Scout appears.
	var scout_id := -1
	for i in MAX_TURNS:
		turn = _main.world.turn
		for unit_id in _main.world.idle_units.duplicate():
			var unit: Dictionary = _main.world.unit_by_id(unit_id)
			if unit["type"] == "unit.scout":
				scout_id = unit_id
				break
			_main._issue_order(unit_id, "Fortify", Vector2i(-1, -1))
		if scout_id >= 0:
			break
		await _wait(func() -> bool: return _main.world.idle_units.is_empty())
		_main._on_ready_pressed({})
		if not await _wait(func() -> bool: return _main.world.turn > turn):
			check(false, "turn %d advances" % turn)
			break
	var progress: Dictionary = _main.world.city_by_id(capital["id"])
	print("turn %d: queue %s, production %d" % [_main.world.turn, progress.get("queue", []), progress.get("unit_production", 0)])
	check(scout_id >= 0, "the produced Scout appears as an idle own unit (id %d, turn %d)" % [scout_id, _main.world.turn])
	if scout_id >= 0:
		check(_main.world.city_by_id(capital["id"])["queue"].is_empty(), "the queue is empty once the unit is produced")
		check(_main._agenda._ready_button.disabled, "the new Scout gates Pronto until it gets an order")
		_main._issue_order(scout_id, "Explore", Vector2i(-1, -1))
		check(await _wait(func() -> bool: return _main.world.idle_units.is_empty()), "the Scout accepts an Explore order")
		turn = _main.world.turn
		_main._on_ready_pressed({})
		check(await _wait(func() -> bool: return _main.world.turn > turn), "the turn resolves with the Scout exploring (now %d)" % _main.world.turn)
	_finish()


func _finish() -> void:
	print("%d e2e checks, %d failures" % [_checks, _failures])
	quit(1 if _failures > 0 else 0)


func _frames(count: int) -> void:
	for i in count:
		await process_frame


func _wait(condition: Callable) -> bool:
	for i in MAX_FRAMES:
		if condition.call():
			return true
		await process_frame
	return condition.call()

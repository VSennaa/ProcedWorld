extends SceneTree
## End-to-end check against a real pw-server (not run in CI: needs a server).
##   PW_E2E_URL=ws://127.0.0.1:18100/ws godot --headless --path client -s res://tests/real_server.gd
## Creates a world, orders every idle unit (Fortify, or Explore for exploration units),
## marks Pronto and expects the turn to advance, for a few turns.

const MAX_FRAMES := 3000
const TURNS := 3

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
	check(not _main.world.units.is_empty(), "own units are visible")
	for i in TURNS:
		var turn: int = _main.world.turn
		var idle: Array = _main.world.idle_units.duplicate()
		print("turn %d: %d units, idle %s" % [turn, _main.world.units.size(), idle])
		if not idle.is_empty():
			check(_main._agenda._ready_button.disabled, "turn %d: Pronto gated while %d units wait" % [turn, idle.size()])
		for unit_id in idle:
			var unit: Dictionary = _main.world.unit_by_id(unit_id)
			var kind := "Explore" if unit.get("role", "") == "exploration" else "Fortify"
			_main._issue_order(unit_id, kind, Vector2i(-1, -1))
		var cleared := await _wait(func() -> bool: return _main.world.idle_units.is_empty())
		check(cleared, "turn %d: every idle unit got an order" % turn)
		_main._on_ready_pressed({})
		var advanced := await _wait(func() -> bool: return _main.world.turn > turn)
		check(advanced, "turn %d: Pronto advances the turn (now %d)" % [turn, _main.world.turn])
		if not advanced:
			break
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

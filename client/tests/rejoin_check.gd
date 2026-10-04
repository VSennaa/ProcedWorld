extends SceneTree
## Manual check (not in CI): after a server restart with PW_DATA_DIR, the last e2e world can be rejoined.
##   PW_E2E_URL=ws://127.0.0.1:18100/ws godot --headless --path client -s res://tests/rejoin_check.gd


func _initialize() -> void:
	_run()


func _run() -> void:
	var main: Node = load("res://scenes/main.tscn").instantiate()
	root.add_child(main)
	for i in 2:
		await process_frame
	main._on_join_requested(OS.get_environment("PW_E2E_URL"), 20261003, 0)
	for i in 3000:
		if main.mode == main.Mode.LIVE and main.world != null and main.world.live:
			break
		await process_frame
	var ok: bool = main.world != null and main.world.live and main.world.turn > 0
	print("rejoin %s: turn %d, cities %d" % ["ok" if ok else "FAIL", main.world.turn if main.world else -1, main.world.own_cities().size() if main.world else -1])
	quit(0 if ok else 1)

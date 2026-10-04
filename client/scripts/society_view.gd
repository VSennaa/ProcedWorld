extends MarginContainer
## Read-only societal indicators supplied in the server projection.
## Pressure has named factors instead of an opaque bar.

const MUTED := Color("b9c1c8")
const CIVILIZATION_ROWS := [["C", "Coesão social", "coesao", "0–100"], ["L", "Legitimidade", "legitimidade", "0–100"], ["D", "Privação", "deprivation", "0–20"], ["G", "Tensão de grupos", "group_tension", "0–20"], ["W", "Ameaça de guerra", "war_threat", "0–20 · provisório"], ["E", "Exposição ambiental", "environmental_exposure", "0–20 · provisório"]]
const CITY_ROWS := [["S", "Estabilidade local", "stability", "0–100"], ["D", "Privação", "deprivation", "0–20"], ["G", "Tensão de grupos", "group_tension", "0–20"], ["W", "Ameaça de guerra", "war_threat", "0–20 · provisório"], ["E", "Exposição ambiental", "environmental_exposure", "0–20 · provisório"]]


func build(world: RefCounted) -> void:
	for child in get_children():
		child.queue_free()
	for side in ["left", "right", "top", "bottom"]:
		add_theme_constant_override("margin_" + side, 12)
	var scroll := ScrollContainer.new()
	scroll.set_anchors_preset(Control.PRESET_FULL_RECT)
	add_child(scroll)
	var root := VBoxContainer.new()
	root.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	root.add_theme_constant_override("separation", 10)
	scroll.add_child(root)
	_add_title(root, "Sociedade")
	_add_note(root, "Indicadores da visão atual do servidor. W e E são provisórios.")
	_add_note(root, "P_c = limitar(0–100, 2D + 2G + W + E + (100 − S)/10 − (C − 50)/5). Fatores: D, G, W e E: 0–20; S e C: 0–100. P_civ é a média de P_c ponderada pela população.")
	_add_title(root, "Civilização")
	for row in CIVILIZATION_ROWS:
		_add_row(root, row, world.indicators)
	_add_pressure(root, world.indicators, "civilização")
	for city in world.own_cities():
		_add_title(root, city["name"])
		var values: Dictionary = city.get("indicators", {})
		for row in CITY_ROWS:
			_add_row(root, row, values)
		_add_pressure(root, values, city["name"], int(world.indicators.get("coesao", 0)))


func _add_title(parent: Control, text: String) -> void:
	var label := Label.new()
	label.text = text
	label.add_theme_font_size_override("font_size", 28 if text == "Sociedade" else 22)
	parent.add_child(label)


func _add_note(parent: Control, text: String) -> void:
	var label := Label.new()
	label.text = text
	label.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	label.add_theme_font_size_override("font_size", 17)
	label.add_theme_color_override("font_color", MUTED)
	parent.add_child(label)


func _add_row(parent: Control, row: Array, values: Dictionary) -> void:
	var label := Label.new()
	label.text = "%s · %s: %d (%s)" % [row[0], row[1], int(values.get(row[2], 0)), row[3]]
	label.add_theme_font_size_override("font_size", 19)
	parent.add_child(label)


func _add_pressure(parent: Control, values: Dictionary, scope: String, cohesion: int = -1) -> void:
	var label := Label.new()
	var factors := "D %d · G %d · W %d · E %d" % [int(values.get("deprivation", 0)), int(values.get("group_tension", 0)), int(values.get("war_threat", 0)), int(values.get("environmental_exposure", 0))]
	if values.has("stability"):
		factors += " · S %d" % int(values["stability"])
	var c := int(values.get("coesao", cohesion)) if cohesion < 0 else cohesion
	if c >= 0:
		factors += " · C %d (−(%d−50)/5 = %+d)" % [c, c, -int((c - 50) / 5)]
	label.text = "P · Pressão de crise (%s): %d/100\nFatores recebidos: %s" % [scope, int(values.get("crisis_pressure", 0)), factors]
	label.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	label.add_theme_font_size_override("font_size", 18)
	label.add_theme_color_override("font_color", MUTED)
	parent.add_child(label)

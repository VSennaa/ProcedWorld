extends PanelContainer
## Card for the selected unit: type, hit points, movement, current order and the order actions.
## Foreign units show information only. The server validates every order; the client only offers
## the actions that make sense for the unit role.

signal action_requested(action: String)  # base action or "attack:<unit>" / "build:<improvement>"

const MUTED_COLOR := Color("b9c1c8")
const ACTION_LABELS := {"move": "Mover", "explore": "Explorar", "fortify": "Fortificar", "sentry": "Prontidão", "found": "Fundar cidade", "skip": "Pular", "attack": "Atacar"}
const NO_FORTIFY_ROLES: Array[String] = ["settler", "worker", "trade"]

var _title: Label
var _details: Label
var _hint: Label
var _buttons: Dictionary = {}
var _row: HBoxContainer
var _choices: VBoxContainer
var _unit_id := -1


func _ready() -> void:
	var style := StyleBoxFlat.new()
	style.bg_color = Color(0.063, 0.094, 0.125, 0.94)
	style.border_color = Color("e69f00")
	style.border_width_top = 3
	style.set_content_margin_all(10)
	add_theme_stylebox_override("panel", style)
	var box := VBoxContainer.new()
	box.add_theme_constant_override("separation", 4)
	add_child(box)
	_title = Label.new()
	_title.add_theme_font_size_override("font_size", 24)
	box.add_child(_title)
	_details = Label.new()
	_details.add_theme_font_size_override("font_size", 19)
	_details.add_theme_color_override("font_color", MUTED_COLOR)
	_details.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	box.add_child(_details)
	_hint = Label.new()
	_hint.add_theme_font_size_override("font_size", 19)
	_hint.add_theme_color_override("font_color", Color("e69f00"))
	_hint.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	_hint.visible = false
	box.add_child(_hint)
	_row = HBoxContainer.new()
	_row.add_theme_constant_override("separation", 8)
	box.add_child(_row)
	_choices = VBoxContainer.new()
	_choices.add_theme_constant_override("separation", 5)
	box.add_child(_choices)
	for action in ACTION_LABELS:
		var button := Button.new()
		button.text = ACTION_LABELS[action]
		button.custom_minimum_size = Vector2(0, 56)
		button.size_flags_horizontal = Control.SIZE_EXPAND_FILL
		button.add_theme_font_size_override("font_size", 20)
		button.pressed.connect(func() -> void: action_requested.emit(action))
		_row.add_child(button)
		_buttons[action] = button
	visible = false


## Actions offered for a unit. Own units only; the server remains the judge of legality.
## `tile` (optional) is what the client knows about the unit's tile: a settler is only offered
## "Fundar cidade" on dry land without a city, and not again once a founding was accepted.
static func legal_actions(unit: Dictionary, tile: Dictionary = {}, has_attack_target: bool = false) -> Array[String]:
	var result: Array[String] = []
	if not unit.get("own", false):
		return result
	result.append("move")
	if unit.get("role", "") == "exploration":
		result.append("explore")
	if not NO_FORTIFY_ROLES.has(unit.get("role", "")):
		result.append("fortify")
		result.append("sentry")
	if unit.get("role", "") == "settler" and not unit.get("founding", false) and not tile.has("city") and tile.get("biome", "") != "oceano":
		result.append("found")
	if has_attack_target:
		result.append("attack")
	result.append("skip")
	return result


static func order_label(unit: Dictionary) -> String:
	if unit.get("founding", false):
		return "fundando uma cidade"
	match String(unit.get("order", "")):
		"Idle":
			return "aguardando ordem"
		"Fortify":
			return "fortificada"
		"Sentry":
			return "em prontidão"
		"Explore":
			return "explorando"
		"MoveTo":
			var target: Vector2i = unit.get("order_target", Vector2i(-1, -1))
			return "indo para (%d, %d)" % [target.x, target.y]
		"Build":
			return "construindo %s" % String(unit.get("order_improvement", "melhoria"))
	return "ordem desconhecida"


func show_unit(unit: Dictionary, turn: int, tile: Dictionary = {}, attack_targets: Array = [], improvement_options: Array = []) -> void:
	_unit_id = unit["id"]
	var own: bool = unit["own"]
	_title.text = "%s%s" % [unit["name"], "" if own else " (de outra civilização)"]
	var movement := "%d" % unit["movement_left"]
	if unit.get("movement_max", 0) > 0:
		movement += "/%d" % unit["movement_max"]
	var line := "Vida %d · movimento %s" % [unit["hp"], movement]
	if own:
		line += " · %s" % order_label(unit)
		if unit.get("skipped_turn", -1) == turn:
			line += " (pulada neste turno)"
	_details.text = line
	var legal := legal_actions(unit, tile, not attack_targets.is_empty())
	for action in _buttons:
		_buttons[action].visible = legal.has(action)
		_buttons[action].text = ACTION_LABELS[action]
	_row.visible = not legal.is_empty()
	for child in _choices.get_children():
		child.queue_free()
	for target in attack_targets:
		_add_choice("Atacar %s" % String(target.get("name", "unidade inimiga")), "attack:%d" % int(target.get("id", -1)))
	for improvement in improvement_options:
		_add_choice("Construir: %s" % String(improvement.get("name", improvement.get("id", "melhoria"))), "build:%s" % String(improvement.get("id", "")))
	_hint.visible = false
	visible = true


func _add_choice(label: String, action: String) -> void:
	var button := Button.new()
	button.text = label
	button.custom_minimum_size = Vector2(0, 48)
	button.add_theme_font_size_override("font_size", 19)
	button.pressed.connect(func() -> void: action_requested.emit(action))
	_choices.add_child(button)


func unit_id() -> int:
	return _unit_id


## While choosing a destination the "Mover" button becomes "Cancelar".
func set_move_mode(on: bool) -> void:
	_buttons["move"].text = "Cancelar" if on else ACTION_LABELS["move"]
	_hint.text = "Toque no destino no mapa."
	_hint.visible = on


func clear() -> void:
	_unit_id = -1
	visible = false

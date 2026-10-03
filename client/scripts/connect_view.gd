extends MarginContainer
## Start screen: server URL, create a world or join one, plus the offline demonstrations.

signal create_requested(url: String, seed: int, civs: int)
signal join_requested(url: String, world_id: int, civ: int)
signal demo_requested(kind: String)  # "pauta" (hand-written fixture) | "servidor" (server-shaped fixture)

const MUTED_COLOR := Color("b9c1c8")
const ERROR_COLOR := Color("ff8a65")
const FIELD_HEIGHT := 56
const MAX_SPIN := 9007199254740991.0  # 2^53 - 1: largest integer a JSON number carries exactly

var _url: LineEdit
var _seed: SpinBox
var _civs: SpinBox
var _world: SpinBox
var _civ: SpinBox
var _status: Label
var _buttons: Array[Button] = []


func build(default_url: String, world_id: int, civ: int) -> void:
	for child in get_children():
		child.queue_free()
	_buttons.clear()
	for side in ["left", "right", "top", "bottom"]:
		add_theme_constant_override("margin_" + side, 20)
	var scroll := ScrollContainer.new()
	scroll.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_DISABLED
	add_child(scroll)
	var column := VBoxContainer.new()
	column.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	column.add_theme_constant_override("separation", 12)
	scroll.add_child(column)

	column.add_child(_label("ProcedWorld", 40, Color("f4f0da")))
	column.add_child(_label("Conecte-se a um servidor ou veja uma demonstração.", 20, MUTED_COLOR))

	column.add_child(_label("Servidor", 24, Color("f4f0da")))
	_url = LineEdit.new()
	_url.text = default_url
	_url.custom_minimum_size = Vector2(0, FIELD_HEIGHT)
	_url.add_theme_font_size_override("font_size", 22)
	column.add_child(_url)

	column.add_child(_label("Criar mundo", 24, Color("f4f0da")))
	_seed = _spin(column, "Semente (id do mundo)", 1, 1, MAX_SPIN)
	_civs = _spin(column, "Civilizações", 4, 1, 16)
	column.add_child(_button("Criar mundo e entrar", _on_create))

	column.add_child(_label("Entrar em um mundo", 24, Color("f4f0da")))
	_world = _spin(column, "Mundo (id)", world_id, 0, MAX_SPIN)
	_civ = _spin(column, "Civilização (começa em 0)", civ, 0, 15)
	column.add_child(_button("Entrar", _on_join))

	column.add_child(_label("Demonstração (sem servidor)", 24, Color("f4f0da")))
	column.add_child(_button("Demonstração: pauta de exemplo", func() -> void: demo_requested.emit("pauta")))
	column.add_child(_button("Demonstração: visão do servidor", func() -> void: demo_requested.emit("servidor")))

	_status = _label("", 20, MUTED_COLOR)
	column.add_child(_status)


func set_status(text: String, is_error: bool = false) -> void:
	if _status != null:
		_status.text = text
		_status.add_theme_color_override("font_color", ERROR_COLOR if is_error else MUTED_COLOR)


func set_busy(busy: bool) -> void:
	for button in _buttons:
		button.disabled = busy


func set_join_target(world_id: int, civ: int) -> void:
	_world.value = world_id
	_civ.value = civ


func _on_create() -> void:
	create_requested.emit(_url.text.strip_edges(), int(_seed.value), int(_civs.value))


func _on_join() -> void:
	join_requested.emit(_url.text.strip_edges(), int(_world.value), int(_civ.value))


func _label(text: String, size: int, color: Color) -> Label:
	var label := Label.new()
	label.text = text
	label.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	label.add_theme_font_size_override("font_size", size)
	label.add_theme_color_override("font_color", color)
	return label


func _button(text: String, callback: Callable) -> Button:
	var button := Button.new()
	button.text = text
	button.custom_minimum_size = Vector2(0, 64)
	button.add_theme_font_size_override("font_size", 22)
	button.pressed.connect(callback)
	_buttons.append(button)
	return button


func _spin(parent: Control, label: String, value: float, min_value: float, max_value: float) -> SpinBox:
	var row := HBoxContainer.new()
	row.add_theme_constant_override("separation", 12)
	var name_label := _label(label, 20, MUTED_COLOR)
	name_label.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	name_label.size_flags_vertical = Control.SIZE_SHRINK_CENTER
	row.add_child(name_label)
	var spin := SpinBox.new()
	spin.min_value = min_value
	spin.max_value = max_value
	spin.step = 1
	spin.value = value
	spin.custom_minimum_size = Vector2(250, FIELD_HEIGHT)
	spin.get_line_edit().add_theme_font_size_override("font_size", 22)
	row.add_child(spin)
	parent.add_child(row)
	return spin

extends MarginContainer
## Pauta screen: up to 1 critical + 2 important decision cards and the "Pronto" button.
## Choices are a local draft; nothing is sent unless main.gd has an open server connection.

signal ready_pressed(choices: Dictionary)
signal unit_focus_requested(unit_id: int)
signal capital_focus_requested(cell: Vector2i)
signal found_capital_requested

const WorldView := preload("res://scripts/world_view.gd")

const CRITICAL_COLOR := Color("d55e00")
const IMPORTANT_COLOR := Color("e69f00")
const TEXT_COLOR := Color("f4f0da")
const MUTED_COLOR := Color("b9c1c8")
const CARD_BG := Color("223140")

var choices: Dictionary = {}  # card id -> option id
var _agenda: Array = []
var _option_buttons: Dictionary = {}  # card id -> {option id: Button}
var _ready_button: Button
var _status: Label
var _pending_label: Label
var _idle_box: VBoxContainer
var _capital_box: VBoxContainer
var _blocked_count := 0
var _sent := false
var _sent_text := ""


func build(agenda: Array, turn: int) -> void:
	for child in get_children():
		child.queue_free()
	choices.clear()
	_option_buttons.clear()
	_agenda = agenda
	add_theme_constant_override("margin_left", 16)
	add_theme_constant_override("margin_right", 16)
	add_theme_constant_override("margin_top", 8)
	add_theme_constant_override("margin_bottom", 8)

	var scroll := ScrollContainer.new()
	scroll.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_DISABLED
	add_child(scroll)
	var column := VBoxContainer.new()
	column.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	column.add_theme_constant_override("separation", 14)
	scroll.add_child(column)

	var heading := Label.new()
	heading.text = "Pauta do turno %d" % turn
	heading.add_theme_font_size_override("font_size", 30)
	column.add_child(heading)
	_capital_box = VBoxContainer.new()
	column.add_child(_capital_box)
	if agenda.is_empty():
		var empty := Label.new()
		empty.text = "Nada exige uma decisão sua neste turno."
		empty.add_theme_color_override("font_color", MUTED_COLOR)
		column.add_child(empty)
	for card in agenda:
		column.add_child(_build_card(card))

	_idle_box = VBoxContainer.new()
	_idle_box.add_theme_constant_override("separation", 8)
	column.add_child(_idle_box)
	_sent = false
	_blocked_count = 0

	_pending_label = Label.new()
	_pending_label.add_theme_color_override("font_color", MUTED_COLOR)
	_pending_label.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	column.add_child(_pending_label)
	_ready_button = Button.new()
	_ready_button.text = "Pronto"
	_ready_button.custom_minimum_size = Vector2(0, 76)
	_ready_button.add_theme_font_size_override("font_size", 28)
	_ready_button.pressed.connect(_on_ready_pressed)
	column.add_child(_ready_button)
	_status = Label.new()
	_status.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	_status.add_theme_color_override("font_color", MUTED_COLOR)
	column.add_child(_status)
	_refresh_pending()


## "Fundar a capital" card, shown while the civilization has no city. `cell` is the starting tile
## ((-1, -1) hides the card); `pending` after the server accepted the founding this turn.
func set_capital_card(cell: Vector2i, pending: bool) -> void:
	if _capital_box == null:
		return
	for child in _capital_box.get_children():
		_capital_box.remove_child(child)
		child.queue_free()
	if cell.x < 0:
		return
	var panel := PanelContainer.new()
	var style := StyleBoxFlat.new()
	style.bg_color = CARD_BG
	style.border_color = CRITICAL_COLOR
	style.border_width_left = 8
	style.set_corner_radius_all(10)
	style.content_margin_left = 20
	style.content_margin_right = 14
	style.content_margin_top = 12
	style.content_margin_bottom = 14
	panel.add_theme_stylebox_override("panel", style)
	var box := VBoxContainer.new()
	box.add_theme_constant_override("separation", 6)
	panel.add_child(box)
	var tag := Label.new()
	tag.text = "CRÍTICA · sem capital"
	tag.add_theme_font_size_override("font_size", 18)
	tag.add_theme_color_override("font_color", CRITICAL_COLOR)
	box.add_child(tag)
	var title := Label.new()
	title.text = "Fundar a capital"
	title.add_theme_font_size_override("font_size", 26)
	box.add_child(title)
	var body := Label.new()
	body.text = "Fundação ordenada: a capital surge quando o turno for resolvido." if pending else "A civilização ainda não tem cidade. O ponto inicial é (%d, %d)." % [cell.x, cell.y]
	body.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	body.add_theme_font_size_override("font_size", 19)
	body.add_theme_color_override("font_color", MUTED_COLOR)
	box.add_child(body)
	var view_button := Button.new()
	view_button.text = "Ver ponto inicial no mapa"
	view_button.custom_minimum_size = Vector2(0, 56)
	view_button.add_theme_font_size_override("font_size", 20)
	view_button.pressed.connect(func() -> void: capital_focus_requested.emit(cell))
	box.add_child(view_button)
	if not pending:
		var found_button := Button.new()
		found_button.text = "Fundar capital"
		found_button.custom_minimum_size = Vector2(0, 64)
		found_button.add_theme_font_size_override("font_size", 22)
		found_button.pressed.connect(func() -> void: found_capital_requested.emit())
		box.add_child(found_button)
	_capital_box.add_child(panel)


func set_status(text: String) -> void:
	if _status != null:
		_status.text = text


func mark_ready(sent: bool) -> void:
	_sent = true
	_sent_text = "Pronto (enviado)" if sent else "Pronto (rascunho local)"
	_refresh_ready()


## Back to editable (the server reopened the turn, or a new turn began).
func reset_ready() -> void:
	_sent = false
	_refresh_ready()


## Lists own idle units that block "Pronto". `entries`: Array of {id, label}. Tapping one emits
## `unit_focus_requested`. An empty list removes the block.
func set_idle_units(entries: Array) -> void:
	if _idle_box == null:
		return
	for child in _idle_box.get_children():
		child.queue_free()
	_blocked_count = entries.size()
	if not entries.is_empty():
		var heading := Label.new()
		heading.text = "Unidades aguardando ordem"
		heading.add_theme_font_size_override("font_size", 24)
		_idle_box.add_child(heading)
		for entry in entries:
			var button := Button.new()
			button.text = entry["label"]
			button.alignment = HORIZONTAL_ALIGNMENT_LEFT
			button.custom_minimum_size = Vector2(0, 56)
			button.add_theme_font_size_override("font_size", 20)
			button.pressed.connect(func() -> void: unit_focus_requested.emit(entry["id"]))
			_idle_box.add_child(button)
	_refresh_ready()


func _refresh_ready() -> void:
	if _ready_button == null:
		return
	if _sent:
		_ready_button.disabled = true
		_ready_button.text = _sent_text
	elif _blocked_count > 0:
		_ready_button.disabled = true
		_ready_button.text = WorldView.gate_text(_blocked_count)
	else:
		_ready_button.disabled = false
		_ready_button.text = "Pronto"


func _build_card(card: Dictionary) -> Control:
	var critical: bool = card["severity"] == "critical"
	var panel := PanelContainer.new()
	var style := StyleBoxFlat.new()
	style.bg_color = CARD_BG
	style.border_color = CRITICAL_COLOR if critical else IMPORTANT_COLOR
	style.border_width_left = 8
	style.set_corner_radius_all(10)
	style.content_margin_left = 20
	style.content_margin_right = 14
	style.content_margin_top = 12
	style.content_margin_bottom = 14
	panel.add_theme_stylebox_override("panel", style)

	var box := VBoxContainer.new()
	box.add_theme_constant_override("separation", 6)
	panel.add_child(box)
	var tag := Label.new()
	tag.text = ("CRÍTICA" if critical else "IMPORTANTE") + " · prazo: %d turnos" % int(card["deadline_turns"])
	tag.add_theme_font_size_override("font_size", 18)
	tag.add_theme_color_override("font_color", CRITICAL_COLOR if critical else IMPORTANT_COLOR)
	box.add_child(tag)
	var title := Label.new()
	title.text = card["title"]
	title.add_theme_font_size_override("font_size", 26)
	title.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	box.add_child(title)
	for pair in [["Causa", "cause"], ["Efeito imediato", "effect"], ["Risco futuro", "risk"]]:
		if String(card.get(pair[1], "")).is_empty():
			continue  # live events only carry what the server sent
		var line := Label.new()
		line.text = "%s: %s" % [pair[0], card[pair[1]]]
		line.add_theme_font_size_override("font_size", 19)
		line.add_theme_color_override("font_color", MUTED_COLOR)
		line.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
		box.add_child(line)

	var buttons := {}
	for option in card["options"]:
		var button := Button.new()
		button.toggle_mode = true
		button.text = "%s — %s" % [option["label"], option["sacrifice"]] if not String(option["sacrifice"]).is_empty() else option["label"]
		button.alignment = HORIZONTAL_ALIGNMENT_LEFT
		button.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
		button.custom_minimum_size = Vector2(0, 64)
		button.add_theme_font_size_override("font_size", 19)
		button.pressed.connect(_on_option_pressed.bind(card["id"], option["id"]))
		box.add_child(button)
		buttons[option["id"]] = button
	_option_buttons[card["id"]] = buttons
	return panel


func _on_option_pressed(card_id: String, option_id: String) -> void:
	choices[card_id] = option_id
	for id in _option_buttons[card_id]:
		_option_buttons[card_id][id].button_pressed = (id == option_id)
	_refresh_pending()


func _refresh_pending() -> void:
	var missing := _agenda.size() - choices.size()
	if missing == 0:
		_pending_label.text = "Todas as decisões escolhidas."
	else:
		_pending_label.text = "%d decisão(ões) sem escolha: o plano atual será mantido." % missing


func _on_ready_pressed() -> void:
	ready_pressed.emit(choices.duplicate())

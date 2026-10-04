extends MarginContainer
## Full, keyboard/touch-accessible detail for one pending Entropy event. The choice remains a
## draft in the Pauta; main.gd submits it only together with Pronto.

signal choice_selected(card_id: String, option_id: String)
signal back_requested

const ServerView := preload("res://scripts/server_view.gd")
const MUTED_COLOR := Color("b9c1c8")
const CARD_BG := Color("223140")


static func deadline_text(card: Dictionary) -> String:
	var turns := int(card.get("deadline_turns", 0))
	return "EVENTO · %s · prazo: %d %s" % [ServerView.category_label(String(card.get("category", ""))), turns, "turno" if turns == 1 else "turnos"]


func show_event(card: Dictionary, selected_choice: String = "") -> void:
	for child in get_children():
		child.queue_free()
	add_theme_constant_override("margin_left", 16)
	add_theme_constant_override("margin_right", 16)
	add_theme_constant_override("margin_top", 12)
	add_theme_constant_override("margin_bottom", 12)
	var scroll := ScrollContainer.new()
	scroll.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_DISABLED
	add_child(scroll)
	var column := VBoxContainer.new()
	column.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	column.add_theme_constant_override("separation", 12)
	scroll.add_child(column)
	var back := Button.new()
	back.text = "Voltar à Pauta"
	back.custom_minimum_size = Vector2(0, 52)
	back.add_theme_font_size_override("font_size", 20)
	back.pressed.connect(func() -> void: back_requested.emit())
	column.add_child(back)
	var panel := PanelContainer.new()
	var style := StyleBoxFlat.new()
	style.bg_color = CARD_BG
	style.border_color = Color("e69f00")
	style.border_width_left = 8
	style.set_corner_radius_all(10)
	style.set_content_margin_all(18)
	panel.add_theme_stylebox_override("panel", style)
	column.add_child(panel)
	var box := VBoxContainer.new()
	box.add_theme_constant_override("separation", 10)
	panel.add_child(box)
	var category := Label.new()
	category.text = deadline_text(card)
	category.add_theme_font_size_override("font_size", 18)
	category.add_theme_color_override("font_color", Color("e69f00"))
	category.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	box.add_child(category)
	var title := Label.new()
	title.text = String(card.get("title", "Evento"))
	title.add_theme_font_size_override("font_size", 30)
	title.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	box.add_child(title)
	var text := String(card.get("effect", ""))
	if not text.is_empty():
		var body := Label.new()
		body.text = text
		body.add_theme_font_size_override("font_size", 21)
		body.add_theme_color_override("font_color", MUTED_COLOR)
		body.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
		box.add_child(body)
	var choices_label := Label.new()
	choices_label.text = "Escolha uma resposta"
	choices_label.add_theme_font_size_override("font_size", 23)
	box.add_child(choices_label)
	for option in card.get("options", []):
		var button := Button.new()
		var suffix := String(option.get("sacrifice", ""))
		button.text = "%s — %s" % [option["label"], suffix] if not suffix.is_empty() else String(option["label"])
		button.toggle_mode = true
		button.button_pressed = String(option.get("id", "")) == selected_choice
		button.alignment = HORIZONTAL_ALIGNMENT_LEFT
		button.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
		button.custom_minimum_size = Vector2(0, 64)
		button.add_theme_font_size_override("font_size", 20)
		button.pressed.connect(func() -> void: choice_selected.emit(String(card["id"]), String(option["id"])))
		box.add_child(button)

extends MarginContainer
## Mandato screen (brief D1b): shows the preset the server reports for this civilization and what
## the Governor will and will not do while the player is absent, with three buttons to switch it.
## Switching only sends `set_mandate`; the screen never changes state on its own, so what it shows
## always comes from the authoritative server view.

signal preset_requested(preset: String)

const MUTED := Color("b9c1c8")
const CARD_BG := Color("223140")

const PRESET_LABELS := {
	"balanced": "Equilibrado",
	"recover": "Recuperar",
	"grow_cautiously": "Crescer com cautela",
}
## What each preset means for a player who is away (GDD 09). These are display texts, not rules.
const PRESET_DESCRIPTIONS := {
	"balanced": "Divide o esforço igualmente entre segurança, sustento, desenvolvimento e relações.",
	"recover": "Prioriza sustento e segurança para reconstruir a sociedade; o desenvolvimento fica em segundo plano.",
	"grow_cautiously": "Prioriza desenvolvimento e sustento, mantendo segurança e relações menores.",
}
const DIRECTION_LABELS := {
	"security": "Segurança", "sustenance": "Sustento",
	"development": "Desenvolvimento", "relations": "Relações",
}
const RED_LINE_LABELS := {
	"no_start_war": "não inicia guerra",
	"no_break_treaty": "não rompe tratado",
	"no_cede_city": "não cede cidade",
	"no_spend_reserve_diplomacy": "não gasta reservas em diplomacia",
	"no_relocate_population": "não desloca população entre cidades",
}
const STANCE_LABELS := {
	"conciliatory": "Conciliadora", "reciprocal": "Recíproca", "deterrent": "Dissuasória",
}
const RESERVE_LEVEL_LABELS := {"zero": "zero", "low": "baixa", "medium": "média", "high": "alta"}
const RESERVE_FIELD_LABELS := {
	"treasury": "Tesouro", "strategic_stock": "Estoque estratégico", "unit_loss": "Perda de unidades",
}

## "Equilibrado" for a known preset, the humanized id otherwise.
static func preset_label(preset: String) -> String:
	if PRESET_LABELS.has(preset):
		return PRESET_LABELS[preset]
	return preset.replace("_", " ").capitalize() if not preset.is_empty() else "desconhecido"


static func preset_description(preset: String) -> String:
	return PRESET_DESCRIPTIONS.get(preset, "Sem descrição para este preset.")


## One line per direction point: "Segurança 20 · Sustento 30 · ...".
static func direction_text(direction: Dictionary) -> String:
	var parts: Array[String] = []
	for key in ["security", "sustenance", "development", "relations"]:
		parts.append("%s %d" % [DIRECTION_LABELS[key], int(direction.get(key, 0))])
	return " · ".join(parts)


## What the Governor never does, from the red lines published in the view.
static func limits_text(limits: Dictionary) -> String:
	var lines: Array[String] = []
	for red_line in limits.get("red_lines", []):
		lines.append(String(RED_LINE_LABELS.get(String(red_line), String(red_line).replace("_", " "))))
	var stance := String(limits.get("stance", ""))
	var suffix := ""
	if STANCE_LABELS.has(stance):
		suffix = " Postura %s." % STANCE_LABELS[stance]
	if lines.is_empty():
		return "Sem linhas vermelhas informadas.%s" % suffix
	return "o Governador %s.%s" % [", ".join(lines), suffix]


func build(world: RefCounted, can_change: bool) -> void:
	for child in get_children():
		child.queue_free()
	for side in ["left", "right", "top", "bottom"]:
		add_theme_constant_override("margin_" + side, 12)
	var scroll := ScrollContainer.new()
	scroll.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_DISABLED
	add_child(scroll)
	var root := VBoxContainer.new()
	root.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	root.add_theme_constant_override("separation", 12)
	scroll.add_child(root)
	_add_title(root, "Mandato")
	_add_note(root, "O Governador administra sua civilização quando você está ausente. O Mandato diz o que ele prioriza e o que nunca pode fazer.")
	var mandate: Dictionary = world.mandate if world != null else {}
	if mandate.is_empty() or String(mandate.get("preset", "")).is_empty():
		_add_note(root, "O servidor ainda não informa o Mandato desta civilização.")
		return

	var preset := String(mandate["preset"])
	root.add_child(_build_current(mandate, preset))

	var pick := Label.new()
	pick.text = "Trocar de mandato"
	pick.add_theme_font_size_override("font_size", 22)
	root.add_child(pick)
	_add_note(root, "A troca é enviada ao servidor; a tela só mostra a visão que ele devolver.")
	if not can_change:
		_add_note(root, "Conecte-se a um servidor para trocar o Mandato. Aqui a tela é só leitura.")
		return
	for option in ["balanced", "recover", "grow_cautiously"]:
		var button := Button.new()
		button.text = "%s — %s" % [preset_label(option), preset_description(option)]
		button.custom_minimum_size = Vector2(0, 52)
		button.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
		button.add_theme_font_size_override("font_size", 19)
		button.disabled = option == preset
		button.pressed.connect(func() -> void: preset_requested.emit(option))
		root.add_child(button)


func _build_current(mandate: Dictionary, preset: String) -> Control:
	var panel := PanelContainer.new()
	var style := StyleBoxFlat.new()
	style.bg_color = CARD_BG
	style.set_corner_radius_all(10)
	style.set_content_margin_all(16)
	panel.add_theme_stylebox_override("panel", style)
	var box := VBoxContainer.new()
	box.add_theme_constant_override("separation", 6)
	panel.add_child(box)
	var title := Label.new()
	title.text = "Mandato atual: %s (versão %d)" % [preset_label(preset), int(mandate.get("version", 0))]
	title.add_theme_font_size_override("font_size", 24)
	title.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	box.add_child(title)
	_add_note(box, preset_description(preset))
	var direction := Label.new()
	direction.text = direction_text(mandate.get("direction", {}))
	direction.add_theme_font_size_override("font_size", 20)
	direction.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	box.add_child(direction)
	var limits := Label.new()
	limits.text = limits_text(mandate.get("limits", {}))
	limits.add_theme_font_size_override("font_size", 19)
	limits.add_theme_color_override("font_color", MUTED)
	limits.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	box.add_child(limits)
	return panel


func _add_title(parent: Control, text: String) -> void:
	var label := Label.new()
	label.text = text
	label.add_theme_font_size_override("font_size", 28)
	parent.add_child(label)


func _add_note(parent: Control, text: String) -> void:
	var label := Label.new()
	label.text = text
	label.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	label.add_theme_font_size_override("font_size", 17)
	label.add_theme_color_override("font_color", MUTED)
	parent.add_child(label)

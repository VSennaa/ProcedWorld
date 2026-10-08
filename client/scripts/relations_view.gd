extends MarginContainer
## Relações screen (brief C2b): the known civilizations with their state in the diplomacy machine,
## the viewer's directional balances (`Cf`/`R`/`Dv` with the GDD 12 scales) and the recent Ledger
## entries between the two, readable in PT-BR. Read-only: proposals come later, so there is no
## button here that would not do anything.

const ServerView := preload("res://scripts/server_view.gd")
const MUTED := Color("b9c1c8")
const CARD_BG := Color("223140")


## "Civilização 2 — Paz"
static func relation_title(relation: Dictionary) -> String:
	return "%s — %s" % [String(relation.get("name", "?")), ServerView.relation_state_label(String(relation.get("state", "")))]


## Directional balances with their canonical scales (GDD 12): Cf 0–100, R 0–100, Dv −100..100.
static func balance_text(relation: Dictionary) -> String:
	return "Cf %d/100 · R %d/100 · Dv %d (−100..100)" % [
		int(relation.get("confidence", 0)), int(relation.get("resentment", 0)), int(relation.get("debt", 0))]


## The pair side a Ledger entry names. The viewer itself reads as "você".
static func counterpart_name(counterpart: int, viewer: int) -> String:
	return "você" if counterpart == viewer else "Civilização %d" % (counterpart + 1)


## "Turno 40 · Tratado (ativa) · contraparte: Civilização 2"
static func ledger_line(entry: Dictionary, viewer: int) -> String:
	return "Turno %d · %s (%s) · contraparte: %s" % [
		int(entry.get("turn", 0)),
		ServerView.ledger_category_label(String(entry.get("category", ""))),
		ServerView.ledger_status_label(String(entry.get("status", ""))),
		counterpart_name(int(entry.get("counterpart", -1)), viewer)]


func build(world: RefCounted) -> void:
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
	_add_title(root, "Relações")
	_add_note(root, "Civilizações conhecidas na visão atual do servidor. Somente leitura: propostas virão depois.")
	_add_note(root, "Por direção: Cf confiança 0–100 · R ressentimento 0–100 · Dv dívida −100..100 (GDD 12).")
	var relations: Array = world.relations
	if relations.is_empty():
		_add_note(root, "Você ainda não encontrou outras civilizações.")
		return
	for relation in relations:
		root.add_child(_build_relation(relation, int(world.civ_id)))


func _build_relation(relation: Dictionary, viewer: int) -> Control:
	var panel := PanelContainer.new()
	var style := StyleBoxFlat.new()
	style.bg_color = CARD_BG
	style.border_color = Color(String(relation.get("color", "#b9c1c8")))
	style.border_width_left = 8
	style.set_corner_radius_all(10)
	style.set_content_margin_all(16)
	panel.add_theme_stylebox_override("panel", style)
	var box := VBoxContainer.new()
	box.add_theme_constant_override("separation", 6)
	panel.add_child(box)
	var title := Label.new()
	title.text = relation_title(relation)
	title.add_theme_font_size_override("font_size", 24)
	title.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	box.add_child(title)
	var balance := Label.new()
	balance.text = balance_text(relation)
	balance.add_theme_font_size_override("font_size", 20)
	box.add_child(balance)
	var ledger: Array = relation.get("ledger", [])
	if ledger.is_empty():
		_add_note(box, "Sem fatos recentes no Ledger.")
		return panel
	var heading := Label.new()
	heading.text = "Últimos fatos"
	heading.add_theme_font_size_override("font_size", 20)
	heading.add_theme_color_override("font_color", MUTED)
	box.add_child(heading)
	for entry in ledger:
		var line := Label.new()
		line.text = ledger_line(entry, viewer)
		line.add_theme_font_size_override("font_size", 19)
		line.add_theme_color_override("font_color", MUTED)
		line.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
		box.add_child(line)
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

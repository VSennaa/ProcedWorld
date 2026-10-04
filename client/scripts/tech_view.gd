extends MarginContainer
## Research tree screen. Only catalog-available technologies can be selected.

signal research_requested(research_id: String)
signal investment_requested(percent: int)

const MUTED_COLOR := Color("b9c1c8")
const CARD_BG := Color("223140")
const STATUS_STYLE := {
	"done": {"color": Color("5fb878"), "label": "Dominada"},
	"current": {"color": Color("e69f00"), "label": "Em pesquisa"},
	"available": {"color": Color("56b4e9"), "label": "Disponível"},
	"locked": {"color": Color("5b6670"), "label": "Bloqueada"},
}
const CARD_WIDTH := 232
const CARD_HEIGHT := 132

var _catalog: RefCounted
var _research: Dictionary = {}


## Rebuilds the screen. `catalog` is a CatalogView; `research` is WorldView.research.
func build(catalog: RefCounted, research: Dictionary) -> void:
	_catalog = catalog
	_research = research
	for child in get_children():
		child.queue_free()
	for side in ["left", "right", "top", "bottom"]:
		add_theme_constant_override("margin_" + side, 12)
	var root := VBoxContainer.new()
	root.add_theme_constant_override("separation", 8)
	add_child(root)

	var heading := Label.new()
	heading.text = "Árvore de pesquisa"
	heading.add_theme_font_size_override("font_size", 30)
	root.add_child(heading)
	var summary := Label.new()
	summary.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	summary.add_theme_font_size_override("font_size", 19)
	summary.add_theme_color_override("font_color", MUTED_COLOR)
	summary.text = summary_text(catalog, research)
	root.add_child(summary)
	var investment_row := HBoxContainer.new()
	var investment_label := Label.new()
	investment_label.text = "Investimento em pesquisa"
	investment_label.add_theme_font_size_override("font_size", 18)
	investment_row.add_child(investment_label)
	var investment := OptionButton.new()
	var percentages: Array = research.get("percentages", [])
	for percent in percentages:
		investment.add_item("%d%%" % int(percent))
		investment.set_item_metadata(investment.item_count - 1, int(percent))
		if int(percent) == int(research.get("investment", 0)):
			investment.select(investment.item_count - 1)
	investment.item_selected.connect(func(index: int) -> void: investment_requested.emit(int(investment.get_item_metadata(index))))
	investment_row.add_child(investment)
	root.add_child(investment_row)
	var investment_note := Label.new()
	investment_note.text = "O investimento usa produção das cidades."
	investment_note.add_theme_font_size_override("font_size", 16)
	investment_note.add_theme_color_override("font_color", MUTED_COLOR)
	root.add_child(investment_note)

	var scroll := ScrollContainer.new()
	scroll.size_flags_vertical = Control.SIZE_EXPAND_FILL
	root.add_child(scroll)
	var row := HBoxContainer.new()
	row.add_theme_constant_override("separation", 14)
	scroll.add_child(row)
	var columns: Array = catalog.columns()
	for depth in columns.size():
		var column := VBoxContainer.new()
		column.add_theme_constant_override("separation", 10)
		var title := Label.new()
		title.text = "Base" if depth == 0 else "Nível %d" % (depth + 1)
		title.add_theme_font_size_override("font_size", 20)
		title.add_theme_color_override("font_color", MUTED_COLOR)
		column.add_child(title)
		for id in columns[depth]:
			column.add_child(_card(id))
		row.add_child(column)


static func summary_text(catalog: RefCounted, research: Dictionary) -> String:
	var done: int = research.get("done", []).size()
	var text := "%d de %d dominadas. " % [done, catalog.techs.size()]
	var current: String = research.get("current", "")
	if current != "" and catalog.techs.has(current):
		var p: Dictionary = catalog.progress(current, research)
		text += "Em pesquisa: %s (%d/%d)." % [catalog.tech_name(current), p["have"], p["cost"]]
	else:
		text += "Nenhuma pesquisa em andamento."
	if current != "" and catalog.techs.has(current):
		var estimate := estimate_turns(catalog, current, research)
		text += " " + ("Estimativa: %d turnos." % estimate if estimate >= 0 else "sem investimento: escolha 10% ou 20%.")
	return text


## An estimate from the rate the current projection exposes; it never changes game state.
static func estimate_turns(catalog: RefCounted, id: String, research: Dictionary) -> int:
	var rate := int(research.get("per_turn", 0))
	if rate <= 0 or not catalog.techs.has(id):
		return -1
	var p: Dictionary = catalog.progress(id, research)
	return ceili(float(maxi(0, p["cost"] - p["have"])) / rate)


func _card(id: String) -> Control:
	var tech: Dictionary = _catalog.techs[id]
	var status: String = _catalog.status(id, _research)
	var style: Dictionary = STATUS_STYLE[status]
	var panel := PanelContainer.new()
	panel.custom_minimum_size = Vector2(CARD_WIDTH, CARD_HEIGHT)
	var box_style := StyleBoxFlat.new()
	box_style.bg_color = CARD_BG
	box_style.border_color = style["color"]
	box_style.set_border_width_all(3)
	box_style.border_width_left = 8
	box_style.set_corner_radius_all(10)
	box_style.set_content_margin_all(10)
	panel.add_theme_stylebox_override("panel", box_style)
	var box := VBoxContainer.new()
	box.add_theme_constant_override("separation", 3)
	panel.add_child(box)

	var name_label := Label.new()
	name_label.text = tech["name"]
	name_label.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	name_label.add_theme_font_size_override("font_size", 22)
	box.add_child(name_label)
	var state_label := Label.new()
	state_label.text = "%s · custo %d" % [style["label"], tech["cost"]]
	state_label.add_theme_font_size_override("font_size", 17)
	state_label.add_theme_color_override("font_color", style["color"])
	box.add_child(state_label)
	if status == "current" or (status != "done" and _catalog.progress(id, _research)["have"] > 0):
		var bar := ProgressBar.new()
		var p: Dictionary = _catalog.progress(id, _research)
		bar.max_value = maxi(p["cost"], 1)
		bar.value = p["have"]
		bar.custom_minimum_size = Vector2(0, 16)
		bar.show_percentage = false
		box.add_child(bar)
		var amount := Label.new()
		amount.text = "%d/%d" % [p["have"], p["cost"]]
		amount.add_theme_font_size_override("font_size", 16)
		box.add_child(amount)
	if status == "locked":
		var missing := Label.new()
		missing.text = "Requer: %s" % ", ".join(_catalog.missing_prerequisites(id, _research))
		missing.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
		missing.add_theme_font_size_override("font_size", 16)
		missing.add_theme_color_override("font_color", MUTED_COLOR)
		box.add_child(missing)
	if status == "available":
		var select := Button.new()
		select.text = "Escolher pesquisa"
		select.custom_minimum_size = Vector2(0, 44)
		select.add_theme_font_size_override("font_size", 17)
		select.pressed.connect(func() -> void: research_requested.emit(id))
		box.add_child(select)
	return panel

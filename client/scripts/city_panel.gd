extends PanelContainer
## Card for a tapped tile that holds an own city, or for the starting tile before the first city:
## population, focus, production queue with progress, the units that can be queued, and the
## "Fundar capital" action. The server validates every command; the client only offers what the
## catalog and the mastered technologies say is possible.

signal queue_requested(unit_type: String)
signal found_capital_requested

const MUTED_COLOR := Color("b9c1c8")
const FOCUS_NAMES := {"supply": "Abastecimento", "build": "Construção", "diversify": "Diversificação"}
const RESOURCE_NAMES := {"production": "produção", "food": "comida", "knowledge": "conhecimento", "wealth": "riqueza", "culture": "cultura"}
const BUTTON_HEIGHT := 56

var _scroll: ScrollContainer
var _box: VBoxContainer
var _city_id := -1


func _ready() -> void:
	_ensure_built()


func _ensure_built() -> void:
	if _box != null:
		return
	var style := StyleBoxFlat.new()
	style.bg_color = Color(0.063, 0.094, 0.125, 0.96)
	style.border_color = Color("56b4e9")
	style.border_width_top = 3
	style.set_content_margin_all(10)
	add_theme_stylebox_override("panel", style)
	_scroll = ScrollContainer.new()
	_scroll.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_DISABLED
	_scroll.custom_minimum_size = Vector2(0, 300)
	add_child(_scroll)
	_box = VBoxContainer.new()
	_box.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	_box.add_theme_constant_override("separation", 6)
	_scroll.add_child(_box)
	visible = false


# ---------------------------------------------------------------- pure helpers (unit-tested)

## "produção 12, conhecimento 2" from a catalog cost dictionary.
static func cost_text(cost: Dictionary) -> String:
	var parts: Array[String] = []
	for key in ["production", "food", "knowledge", "wealth", "culture"]:
		if cost.has(key) and int(cost[key]) > 0:
			parts.append("%s %d" % [RESOURCE_NAMES[key], int(cost[key])])
	return ", ".join(parts) if not parts.is_empty() else "sem custo"


## Unit types the civilization may queue: the required technology is mastered. Each entry is
## {id, name, cost_text}. `research` is WorldView.research ({done: [...]}). Sorted by name.
static func producible_units(catalog: RefCounted, research: Dictionary) -> Array:
	var result: Array = []
	if catalog == null:
		return result
	for id in catalog.unit_types:
		var entry: Dictionary = catalog.unit_types[id]
		if research.get("done", []).has(String(entry.get("requires_technology", ""))):
			result.append({"id": id, "name": String(entry.get("name", id)), "cost_text": cost_text(entry.get("cost", {}))})
	result.sort_custom(func(a: Dictionary, b: Dictionary) -> bool: return a["name"] < b["name"])
	return result


## Unit types still locked: {name, tech} where `tech` is the display name of the missing technology.
static func locked_units(catalog: RefCounted, research: Dictionary) -> Array:
	var result: Array = []
	if catalog == null:
		return result
	for id in catalog.unit_types:
		var entry: Dictionary = catalog.unit_types[id]
		var tech := String(entry.get("requires_technology", ""))
		if not tech.is_empty() and not research.get("done", []).has(tech):
			result.append({"name": String(entry.get("name", id)), "tech": catalog.tech_name(tech)})
	result.sort_custom(func(a: Dictionary, b: Dictionary) -> bool: return a["name"] < b["name"])
	return result


static func unit_name(catalog: RefCounted, unit_type: String) -> String:
	if catalog != null and catalog.unit_types.has(unit_type):
		return String(catalog.unit_types[unit_type].get("name", unit_type))
	return unit_type.get_slice(".", unit_type.get_slice_count(".") - 1).capitalize()


## "Batedor: produção 4/12 (+2 por turno)" for the head of the queue; "" when the queue is empty.
static func progress_text(city: Dictionary, catalog: RefCounted) -> String:
	var queue: Array = city.get("queue", [])
	if queue.is_empty():
		return ""
	var line := unit_name(catalog, queue[0])
	var cost: Dictionary = {}
	if catalog != null and catalog.unit_types.has(queue[0]):
		cost = catalog.unit_types[queue[0]].get("cost", {})
	var have := int(city.get("unit_production", 0))
	if cost.has("production"):
		line += ": produção %d/%d" % [have, int(cost["production"])]
	else:
		line += ": produção %d" % have
	line += " (+%d por turno)" % int(city.get("yields", {}).get("production", 0))
	return line


static func summary_text(city: Dictionary) -> String:
	return "População %d/%d · foco: %s · comida em estoque %d · estabilidade %d" % [
		int(city["population"]), int(city["housing"]), FOCUS_NAMES.get(city["focus"], "desconhecido"),
		int(city["food_stock"]), int(city["stability"])]


# ---------------------------------------------------------------- display

func city_id() -> int:
	return _city_id


func show_city(city: Dictionary, catalog: RefCounted, research: Dictionary) -> void:
	_ensure_built()
	_clear_box()
	_city_id = city["id"]
	_add_label(city["name"], 24, Color.WHITE)
	_add_label(summary_text(city), 19, MUTED_COLOR)
	_add_label("Fila de produção", 21, Color.WHITE)
	var queue: Array = city["queue"]
	if queue.is_empty():
		_add_label("Vazia: escolha uma unidade para produzir.", 19, MUTED_COLOR)
	else:
		_add_label(progress_text(city, catalog), 19, Color("e69f00"))
		for i in range(1, queue.size()):
			_add_label("%d. %s" % [i + 1, unit_name(catalog, queue[i])], 19, MUTED_COLOR)
	var options := producible_units(catalog, research)
	if catalog == null:
		_add_label("Catálogo ainda não recebido: as unidades aparecem em instantes.", 19, MUTED_COLOR)
	elif options.is_empty():
		_add_label("Nenhuma unidade disponível: pesquise a tecnologia que a libera.", 19, MUTED_COLOR)
	for option in options:
		var button := _add_button("Produzir %s (%s)" % [option["name"], option["cost_text"]])
		var unit_type: String = option["id"]
		button.pressed.connect(func() -> void: queue_requested.emit(unit_type))
	for locked in locked_units(catalog, research):
		_add_label("%s exige %s." % [locked["name"], locked["tech"]], 17, MUTED_COLOR)
	visible = true


## The starting tile before the first city exists (`pending`: a founding was already accepted).
func show_founding(cell: Vector2i, pending: bool) -> void:
	_ensure_built()
	_clear_box()
	_city_id = -1
	_add_label("Ponto inicial (%d, %d)" % [cell.x, cell.y], 24, Color.WHITE)
	if pending:
		_add_label("Fundação ordenada: a capital surge quando o turno for resolvido.", 19, Color("e69f00"))
	else:
		_add_label("Aqui a civilização começa. Sem capital não há produção nem crescimento.", 19, MUTED_COLOR)
		var button := _add_button("Fundar capital")
		button.pressed.connect(func() -> void: found_capital_requested.emit())
	visible = true


func clear() -> void:
	_city_id = -1
	visible = false


func _clear_box() -> void:
	for child in _box.get_children():
		_box.remove_child(child)
		child.queue_free()


func _add_label(text: String, size: int, color: Color) -> Label:
	var label := Label.new()
	label.text = text
	label.add_theme_font_size_override("font_size", size)
	label.add_theme_color_override("font_color", color)
	label.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	_box.add_child(label)
	return label


func _add_button(text: String) -> Button:
	var button := Button.new()
	button.text = text
	button.custom_minimum_size = Vector2(0, BUTTON_HEIGHT)
	button.add_theme_font_size_override("font_size", 20)
	button.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	_box.add_child(button)
	return button

extends Control
## App shell: header (turn + resources), one screen at a time, bottom navigation bar.
## Data comes from res://fixtures/state_snapshot.json unless started with `-- --connect`.

const WorldView := preload("res://scripts/world_view.gd")
const AgendaView := preload("res://scripts/agenda_view.gd")
const MapView := preload("res://scripts/map_view.gd")
const NetClient := preload("res://scripts/net_client.gd")
const Hex := preload("res://scripts/hex.gd")

const FIXTURE_PATH := "res://fixtures/state_snapshot.json"
const BG_COLOR := Color("17212b")
const TEXT_COLOR := Color("f4f0da")
const BIOME_NAMES := {
	"oceano": "Oceano", "costa": "Costa", "planicie": "Planície", "floresta": "Floresta",
	"selva": "Selva", "savana": "Savana", "deserto": "Deserto", "estepe": "Estepe",
	"tundra": "Tundra", "pantano": "Pântano", "montanha": "Montanha", "geleira": "Geleira",
}
const SCREENS: Array[Array] = [
	["pauta", "Pauta"], ["mapa", "Mapa"], ["sociedade", "Sociedade"], ["relacoes", "Relações"], ["cronica", "Crônica"],
]
const PLACEHOLDERS := {
	"sociedade": "O que sustenta a civilização: coesão, crises e focos permitidos.",
	"relacoes": "Em quem confiar e o que é devido: ledger, tratados e propostas.",
	"cronica": "O que aconteceu e quanto custou a IA: fatos, narrativa e consumo.",
}

var world: RefCounted
var net: Node
var _screens: Dictionary = {}
var _nav_buttons: Dictionary = {}
var _header: Label
var _resource_box: HBoxContainer
var _tile_info: Label
var _agenda
var _map


func _ready() -> void:
	var args := OS.get_cmdline_user_args()
	var loaded := WorldView.load_fixture(FIXTURE_PATH)
	_build_shell()
	if not loaded["ok"]:
		_show_error("Falha ao carregar o estado: %s" % loaded["error"])
		return
	world = loaded["view"]
	_populate()
	if "--connect" in args:
		net = NetClient.new()
		add_child(net)
		net.connect_to_server()
	var start := "pauta"
	for arg in args:
		if arg.begins_with("--screen="):
			start = arg.trim_prefix("--screen=")
	show_screen(start)
	if start == "mapa" and "--select-capital" in args:
		_map.select_cell(world.capital)


func _build_shell() -> void:
	set_anchors_preset(Control.PRESET_FULL_RECT)
	var bg := ColorRect.new()
	bg.color = BG_COLOR
	bg.set_anchors_preset(Control.PRESET_FULL_RECT)
	add_child(bg)
	var root := VBoxContainer.new()
	root.set_anchors_preset(Control.PRESET_FULL_RECT)
	root.add_theme_constant_override("separation", 0)
	add_child(root)

	var top := PanelContainer.new()
	top.add_theme_stylebox_override("panel", _flat(Color("101820"), 12))
	var top_box := VBoxContainer.new()
	top.add_child(top_box)
	_header = Label.new()
	_header.add_theme_font_size_override("font_size", 22)
	top_box.add_child(_header)
	_resource_box = HBoxContainer.new()
	_resource_box.add_theme_constant_override("separation", 18)
	top_box.add_child(_resource_box)
	root.add_child(top)

	var content := Control.new()
	content.size_flags_vertical = Control.SIZE_EXPAND_FILL
	root.add_child(content)

	_agenda = AgendaView.new()
	_agenda.set_anchors_preset(Control.PRESET_FULL_RECT)
	_agenda.ready_pressed.connect(_on_ready_pressed)
	content.add_child(_agenda)
	_screens["pauta"] = _agenda

	var map_holder := Control.new()
	map_holder.set_anchors_preset(Control.PRESET_FULL_RECT)
	content.add_child(map_holder)
	_map = MapView.new()
	_map.set_anchors_preset(Control.PRESET_FULL_RECT)
	_map.cell_selected.connect(_on_cell_selected)
	map_holder.add_child(_map)
	_tile_info = Label.new()
	_tile_info.set_anchors_preset(Control.PRESET_BOTTOM_WIDE)
	_tile_info.offset_top = -64
	_tile_info.offset_left = 12
	_tile_info.add_theme_font_size_override("font_size", 20)
	_tile_info.add_theme_color_override("font_shadow_color", Color.BLACK)
	_tile_info.add_theme_constant_override("shadow_offset_x", 2)
	_tile_info.add_theme_constant_override("shadow_offset_y", 2)
	_tile_info.text = "Arraste para mover · pinça ou roda para zoom · toque em um hexágono"
	map_holder.add_child(_tile_info)
	_screens["mapa"] = map_holder

	for key in PLACEHOLDERS:
		_screens[key] = _build_placeholder(key)
		content.add_child(_screens[key])

	var bar := HBoxContainer.new()
	bar.custom_minimum_size = Vector2(0, 88)
	bar.add_theme_constant_override("separation", 2)
	for entry in SCREENS:
		var button := Button.new()
		button.text = entry[1]
		button.toggle_mode = true
		button.size_flags_horizontal = Control.SIZE_EXPAND_FILL
		button.add_theme_font_size_override("font_size", 19)
		button.pressed.connect(show_screen.bind(entry[0]))
		bar.add_child(button)
		_nav_buttons[entry[0]] = button
	root.add_child(bar)


func _build_placeholder(key: String) -> Control:
	var center := CenterContainer.new()
	center.set_anchors_preset(Control.PRESET_FULL_RECT)
	var box := VBoxContainer.new()
	box.custom_minimum_size = Vector2(560, 0)
	center.add_child(box)
	var title := Label.new()
	title.text = "%s — em breve" % _screen_title(key)
	title.add_theme_font_size_override("font_size", 32)
	title.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
	box.add_child(title)
	var body := Label.new()
	body.text = "%s\nEsta tela ainda não existe no cliente: nenhuma informação aqui é real." % PLACEHOLDERS[key]
	body.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	body.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
	body.add_theme_font_size_override("font_size", 20)
	body.add_theme_color_override("font_color", Color("b9c1c8"))
	box.add_child(body)
	return center


func _screen_title(key: String) -> String:
	for entry in SCREENS:
		if entry[0] == key:
			return entry[1]
	return key


func _populate() -> void:
	_header.text = "Turno %d · %s · %s" % [world.turn, world.era, world.civilization.get("name", "?")]
	for child in _resource_box.get_children():
		child.queue_free()
	for resource in ["comida", "producao", "metal", "luxo"]:
		var icon := TextureRect.new()
		icon.texture = load("res://assets/icons/%s.svg" % resource)
		icon.custom_minimum_size = Vector2(28, 28)
		icon.expand_mode = TextureRect.EXPAND_IGNORE_SIZE
		icon.stretch_mode = TextureRect.STRETCH_KEEP_ASPECT_CENTERED
		var label := Label.new()
		label.text = str(int(world.resources.get(resource, 0)))
		label.add_theme_font_size_override("font_size", 20)
		_resource_box.add_child(icon)
		_resource_box.add_child(label)
	_agenda.build(world.agenda, world.turn)
	_agenda.set_status("Dados do arquivo de exemplo (fixtures); sem conexão com o servidor." if net == null else "")
	_map.set_view(world)


func show_screen(key: String) -> void:
	for id in _screens:
		_screens[id].visible = (id == key)
	for id in _nav_buttons:
		_nav_buttons[id].button_pressed = (id == key)


func _on_cell_selected(cell: Vector2i) -> void:
	if cell.x < 0:
		_tile_info.text = "Fora do mapa"
		return
	var tile: Dictionary = world.tile_at(cell)
	if tile["fog"] == "unknown":
		_tile_info.text = "(%d, %d) · território desconhecido" % [cell.x, cell.y]
		return
	var parts: Array[String] = ["(%d, %d)" % [cell.x, cell.y], BIOME_NAMES.get(tile["biome"], tile["biome"])]
	if tile.has("city"):
		parts.append("cidade de %s" % tile["city"]["name"])
	if tile.has("resource"):
		parts.append("recurso: %s" % tile["resource"])
	parts.append("visível" if tile["fog"] == "visible" else "lembrado")
	_tile_info.text = " · ".join(parts)


func _on_ready_pressed(choices: Dictionary) -> void:
	if net != null and net.is_open():
		net.send_message("ready_set", {"turn": world.turn, "ready": true, "choices": choices})
		_agenda.mark_ready(true)
		_agenda.set_status("Enviado ao servidor.")
	else:
		_agenda.mark_ready(false)
		_agenda.set_status("Sem conexão com o servidor: nada foi enviado. Escolhas ficaram apenas neste rascunho local.")


func _show_error(message: String) -> void:
	var label := Label.new()
	label.text = message
	label.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	label.set_anchors_preset(Control.PRESET_FULL_RECT)
	add_child(label)


func _flat(color: Color, margin: int) -> StyleBoxFlat:
	var style := StyleBoxFlat.new()
	style.bg_color = color
	style.set_content_margin_all(margin)
	return style

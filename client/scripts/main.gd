extends Control
## App shell: start screen (server / demonstration), header (turn + resources), one screen at a
## time, bottom navigation bar. Live mode talks to the authoritative server; demonstration modes
## use the fixtures in res://fixtures and never touch the network.
##
## Startup arguments (after `--`): --demo=pauta|servidor  --server=<ws url>  --screen=<id>
## --select-capital  --select-unit=<id>  --no-city (servidor demo: world without cities)
## --select-city=<id> (servidor demo: open the city card)

const WorldView := preload("res://scripts/world_view.gd")
const ServerView := preload("res://scripts/server_view.gd")
const CatalogView := preload("res://scripts/catalog_view.gd")
const SessionStore := preload("res://scripts/session_store.gd")
const Protocol := preload("res://scripts/protocol.gd")
const Hex := preload("res://scripts/hex.gd")
const AgendaView := preload("res://scripts/agenda_view.gd")
const MapView := preload("res://scripts/map_view.gd")
const TechView := preload("res://scripts/tech_view.gd")
const UnitPanel := preload("res://scripts/unit_panel.gd")
const CityPanel := preload("res://scripts/city_panel.gd")
const ConnectView := preload("res://scripts/connect_view.gd")
const NetClient := preload("res://scripts/net_client.gd")

const FIXTURE_PATH := "res://fixtures/state_snapshot.json"
const SERVER_VIEW_PATH := "res://fixtures/server_view.json"
const CATALOG_PATH := "res://fixtures/server_catalog.json"
const BG_COLOR := Color("17212b")
const TEXT_COLOR := Color("f4f0da")
const BIOME_NAMES := {
	"oceano": "Oceano", "costa": "Costa", "planicie": "Planície", "floresta": "Floresta",
	"selva": "Selva", "savana": "Savana", "deserto": "Deserto", "estepe": "Estepe",
	"tundra": "Tundra", "pantano": "Pântano", "montanha": "Montanha", "geleira": "Geleira",
}
const SCREENS: Array[Array] = [
	["pauta", "Pauta"], ["mapa", "Mapa"], ["pesquisa", "Pesquisa"], ["sociedade", "Sociedade"],
	["relacoes", "Relações"], ["cronica", "Crônica"],
]
const PLACEHOLDERS := {
	"sociedade": "O que sustenta a civilização: coesão, crises e focos permitidos.",
	"relacoes": "Em quem confiar e o que é devido: ledger, tratados e propostas.",
	"cronica": "O que aconteceu e quanto custou a IA: fatos, narrativa e consumo.",
}
const ENGINE_REASON_TEXT := {
	"CityAlreadyExists": "Esse identificador de cidade já existe.",
	"CityTileOccupied": "Já existe uma cidade nesse tile.",
	"MissingSettler": "É preciso um colono nesse tile para fundar uma cidade.",
	"UnitTechnologyNotResearched": "A tecnologia dessa unidade ainda não foi dominada.",
	"UnknownCity": "Cidade desconhecida.",
	"NotCommandOwner": "Essa cidade não é sua.",
}
const ERROR_TEXT := {
	"protocol_version_mismatch": "Versão do protocolo incompatível com o servidor.",
	"malformed_message": "O servidor não entendeu a mensagem.",
	"world_not_found": "Mundo não encontrado.",
	"world_exists": "Já existe um mundo com essa semente: use Entrar.",
	"too_many_worlds": "O servidor atingiu o limite de mundos.",
	"invalid_world_params": "Parâmetros de mundo inválidos.",
	"not_joined": "Entre em uma civilização primeiro.",
	"already_joined": "Esta conexão já ocupa uma civilização.",
	"civ_not_found": "Civilização inexistente neste mundo.",
	"civ_taken": "Civilização já ocupada e sem token guardado para retomá-la.",
	"invalid_session_token": "Token de sessão inválido para esta civilização.",
	"already_ready": "Você já marcou Pronto neste turno.",
	"command_rejected": "O servidor recusou a ordem.",
	"units_awaiting_orders": "Ainda há unidades aguardando ordem: dê uma ordem ou pule.",
	"internal": "Erro interno do servidor.",
}

enum Mode { CONNECT, DEMO, LIVE }

var world: RefCounted
var net: Node
var mode := Mode.CONNECT
var _catalog: RefCounted
var _last_payload: Dictionary = {}
var _url := NetClient.DEFAULT_URL
var _pending_action: Dictionary = {}
var _joined := false
var _session: Dictionary = {}
var _inflight: Dictionary = {}  # request id -> {kind, unit_id, order_kind, order_target, ...}
var _overrides: Array = []      # accepted local orders of the open turn, re-applied on refresh
var _overrides_turn := -1
var _selected_unit := -1
var _selected_city := -1
var _founding_selected := false  # the starting tile (no city yet) is selected
var _move_mode := false
var _fresh := true  # next installed view is the first of a session (recenter, show the Pauta)
var _screens: Dictionary = {}
var _nav_buttons: Dictionary = {}
var _top: Control
var _nav_bar: Control
var _header: Label
var _resource_box: HBoxContainer
var _tile_info: Label
var _connect: Control
var _agenda
var _map
var _tech
var _unit_panel
var _city_panel


func _ready() -> void:
	var args := OS.get_cmdline_user_args()
	_build_shell()
	var last := SessionStore.load_last(_url)
	_url = last["url"]
	for arg in args:
		if arg.begins_with("--server="):
			_url = arg.trim_prefix("--server=")
	_connect.build(_url, last["world_id"], last["civ"])
	var start := "pauta"
	for arg in args:
		if arg.begins_with("--screen="):
			start = arg.trim_prefix("--screen=")
	for arg in args:
		if arg.begins_with("--demo="):
			_start_demo(arg.trim_prefix("--demo="))
			show_screen(start)
			if start == "mapa" and "--select-capital" in args:
				_map.select_cell(world.capital)
			if arg == "--demo=servidor":
				for other in args:
					if other.begins_with("--select-unit="):
						_select_unit(int(other.trim_prefix("--select-unit=")))
					elif other.begins_with("--select-city="):
						_select_city(int(other.trim_prefix("--select-city=")))
					elif other == "--select-home" and world.needs_capital():
						_map.focus_cell(world.home_cell)
						_select_city(-1)
			return
	_show_connect()


# ---------------------------------------------------------------- shell

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
	_top = top
	var top_box := VBoxContainer.new()
	top.add_child(top_box)
	var title_row := HBoxContainer.new()
	top_box.add_child(title_row)
	_header = Label.new()
	_header.add_theme_font_size_override("font_size", 22)
	_header.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	_header.size_flags_vertical = Control.SIZE_SHRINK_CENTER
	_header.clip_text = true
	title_row.add_child(_header)
	var leave := Button.new()
	leave.text = "Sair"
	leave.custom_minimum_size = Vector2(96, 48)
	leave.add_theme_font_size_override("font_size", 20)
	leave.pressed.connect(_on_leave_pressed)
	title_row.add_child(leave)
	_resource_box = HBoxContainer.new()
	_resource_box.add_theme_constant_override("separation", 18)
	top_box.add_child(_resource_box)
	root.add_child(top)

	var content := Control.new()
	content.size_flags_vertical = Control.SIZE_EXPAND_FILL
	root.add_child(content)

	_connect = ConnectView.new()
	_connect.set_anchors_preset(Control.PRESET_FULL_RECT)
	_connect.create_requested.connect(_on_create_requested)
	_connect.join_requested.connect(_on_join_requested)
	_connect.demo_requested.connect(_start_demo)
	content.add_child(_connect)
	_screens["conectar"] = _connect

	_agenda = AgendaView.new()
	_agenda.set_anchors_preset(Control.PRESET_FULL_RECT)
	_agenda.ready_pressed.connect(_on_ready_pressed)
	_agenda.unit_focus_requested.connect(_on_unit_focus_requested)
	_agenda.capital_focus_requested.connect(_on_capital_focus_requested)
	_agenda.found_capital_requested.connect(_on_found_capital_requested)
	content.add_child(_agenda)
	_screens["pauta"] = _agenda

	var map_holder := Control.new()
	map_holder.set_anchors_preset(Control.PRESET_FULL_RECT)
	content.add_child(map_holder)
	_map = MapView.new()
	_map.set_anchors_preset(Control.PRESET_FULL_RECT)
	_map.cell_selected.connect(_on_cell_selected)
	_map.target_chosen.connect(_on_target_chosen)
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
	_unit_panel = UnitPanel.new()
	_unit_panel.anchor_left = 0.0
	_unit_panel.anchor_right = 1.0
	_unit_panel.anchor_top = 1.0
	_unit_panel.anchor_bottom = 1.0
	_unit_panel.offset_bottom = -64
	_unit_panel.offset_top = -64
	_unit_panel.grow_vertical = Control.GROW_DIRECTION_BEGIN
	_unit_panel.action_requested.connect(_on_unit_action)
	map_holder.add_child(_unit_panel)
	_city_panel = CityPanel.new()
	_city_panel.anchor_left = 0.0
	_city_panel.anchor_right = 1.0
	_city_panel.anchor_top = 1.0
	_city_panel.anchor_bottom = 1.0
	_city_panel.offset_bottom = -64
	_city_panel.offset_top = -64
	_city_panel.grow_vertical = Control.GROW_DIRECTION_BEGIN
	_city_panel.queue_requested.connect(_on_queue_requested)
	_city_panel.found_capital_requested.connect(_on_found_capital_requested)
	map_holder.add_child(_city_panel)
	_screens["mapa"] = map_holder

	_tech = TechView.new()
	_tech.set_anchors_preset(Control.PRESET_FULL_RECT)
	content.add_child(_tech)
	_screens["pesquisa"] = _tech

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
		button.add_theme_font_size_override("font_size", 17)
		button.pressed.connect(show_screen.bind(entry[0]))
		bar.add_child(button)
		_nav_buttons[entry[0]] = button
	_nav_bar = bar
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


func show_screen(key: String) -> void:
	for id in _screens:
		_screens[id].visible = (id == key)
	for id in _nav_buttons:
		_nav_buttons[id].button_pressed = (id == key)


func _show_connect() -> void:
	mode = Mode.CONNECT
	_fresh = true
	_selected_unit = -1
	_last_payload = {}
	_catalog = null
	_top.visible = false
	_nav_bar.visible = false
	show_screen("conectar")


func _show_game() -> void:
	_top.visible = true
	_nav_bar.visible = true


# ---------------------------------------------------------------- connect screen and live session

func _on_create_requested(url: String, seed: int, civs: int) -> void:
	_pending_action = {"kind": "create", "seed": seed, "civs": civs}
	_open_connection(url)


func _on_join_requested(url: String, world_id: int, civ: int) -> void:
	_pending_action = {"kind": "join", "world_id": world_id, "civ": civ}
	_open_connection(url)


func _open_connection(url: String) -> void:
	_close_connection()
	_url = url
	_joined = false
	net = NetClient.new()
	add_child(net)
	net.connection_changed.connect(_on_connection_changed)
	net.connection_lost.connect(_on_connection_lost)
	net.envelope_received.connect(_on_envelope)
	net.protocol_error.connect(func(message: String) -> void: _report("Mensagem inválida do servidor: %s" % message, true))
	if url.is_empty() or net.connect_to_server(url) != OK:
		_close_connection()
		_connect.set_status("Endereço do servidor inválido.", true)
		return
	_connect.set_busy(true)
	_connect.set_status("Conectando…")


func _close_connection() -> void:
	if net != null:
		net.queue_free()
		net = null


func _on_connection_changed(is_open: bool) -> void:
	if is_open and not _pending_action.is_empty():
		_run_pending_action()


func _run_pending_action() -> void:
	var action := _pending_action
	if action["kind"] == "create":
		_connect.set_status("Criando o mundo…")
		_inflight[net.send_request("create_world", {"seed": action["seed"], "civs": action["civs"]})] = {"kind": "create"}
	else:
		_connect.set_status("Entrando no mundo…")
		var payload := {"world_id": action["world_id"], "civ": action["civ"]}
		var token := SessionStore.load_token(action["world_id"], action["civ"])
		if not token.is_empty():
			payload["session_token"] = token
		_inflight[net.send_request("join", payload)] = {"kind": "join"}


func _on_connection_lost(was_open: bool) -> void:
	var text := "Conexão com o servidor perdida. Use Entrar para retomar a mesma civilização." if was_open else "Não foi possível conectar a %s." % _url
	_close_connection()
	_inflight.clear()
	var last := SessionStore.load_last(_url)
	_connect.build(_url, last["world_id"], last["civ"])
	_connect.set_status(text, true)
	_show_connect()


func _on_leave_pressed() -> void:
	_close_connection()
	_inflight.clear()
	_pending_action = {}
	_joined = false
	_unit_panel.clear()
	_city_panel.clear()
	_selected_city = -1
	_founding_selected = false
	_set_move_mode(false)
	var last := SessionStore.load_last(_url)
	_connect.build(_url, last["world_id"], last["civ"])
	_show_connect()


func _on_envelope(envelope: Dictionary) -> void:
	var payload: Dictionary = envelope["payload"]
	var request_id: String = envelope["request_id"]
	match String(envelope["type"]):
		"world_created":
			_inflight.erase(request_id)
			var world_id := int(payload.get("world_id", 0))
			_connect.set_join_target(world_id, 0)
			_pending_action = {"kind": "join", "world_id": world_id, "civ": 0}
			_run_pending_action()
		"joined":
			_inflight.erase(request_id)
			_joined = true
			mode = Mode.LIVE
			_session = {"world_id": int(payload.get("world_id", 0)), "civ": int(payload.get("civ", 0))}
			var token := String(payload.get("session_token", ""))
			if not token.is_empty():
				SessionStore.save_token(_session["world_id"], _session["civ"], token)
			SessionStore.save_last(_url, _session["world_id"], _session["civ"])
		"state_snapshot":
			_apply_view(payload, false)
		"turn_diff":
			if typeof(payload.get("view")) == TYPE_DICTIONARY:
				_apply_view(payload["view"], true)
				_report("Turno %d resolvido." % int(payload.get("turn", 0)), false)
		"catalog":
			_apply_catalog(payload)
		"ready_state":
			_apply_ready_state(payload)
		"command_accepted":
			_on_command_accepted(request_id)
		"error":
			_on_error(request_id, payload)


func _apply_catalog(payload: Dictionary) -> void:
	var parsed := CatalogView.from_payload(payload)
	if not parsed["ok"]:
		_report("Catálogo inválido: %s" % parsed["error"], true)
		return
	_catalog = parsed["catalog"]
	if not _last_payload.is_empty():
		_apply_view(_last_payload, false)  # unit names and roles come from the catalog


## Installs a server view. `new_turn` clears orders that were only accepted locally for the old turn.
func _apply_view(payload: Dictionary, new_turn: bool) -> void:
	var parsed := ServerView.from_payload(payload, _catalog)
	if not parsed["ok"]:
		_report("Visão inválida do servidor: %s" % parsed["error"], true)
		return
	var first := _fresh
	_fresh = false
	_last_payload = payload
	world = parsed["view"]
	if new_turn or _overrides_turn != world.turn:
		_overrides.clear()
		_overrides_turn = world.turn
	for entry in _overrides:  # accepted this turn but maybe not yet reflected in the server view
		_apply_override(entry)
	_enter_game(first)


func _enter_game(first: bool) -> void:
	_show_game()
	_populate(first)
	if first:
		show_screen("pauta")


func _apply_ready_state(payload: Dictionary) -> void:
	if world == null:
		return
	# JSON numbers arrive as floats; compare as ints.
	var ready: Array = payload.get("ready", []).map(func(id: Variant) -> int: return int(id))
	var present: Array = payload.get("present", [])
	if ready.has(world.civ_id):
		_agenda.mark_ready(true)
	else:
		_agenda.reset_ready()
	_agenda.set_status("Prontos: %d de %d." % [ready.size(), present.size()])


func _on_command_accepted(request_id: String) -> void:
	if not _inflight.has(request_id):
		return
	var meta: Dictionary = _inflight[request_id]
	_inflight.erase(request_id)
	if meta["kind"] in ["order", "skip", "found", "queue"]:
		_overrides.append(meta)
		_apply_override(meta)
		_refresh_units()


func _on_error(request_id: String, payload: Dictionary) -> void:
	var meta: Dictionary = _inflight.get(request_id, {})
	_inflight.erase(request_id)
	var reason := String(payload.get("reason", ""))
	var text: String = ERROR_TEXT.get(reason, "Erro do servidor: %s" % reason)
	var detail := String(payload.get("detail", ""))
	if reason == "command_rejected" and payload.get("engine_reason") != null:
		var engine_reason := String(payload["engine_reason"])
		text = ENGINE_REASON_TEXT.get(engine_reason, "%s (%s)" % [text, engine_reason])
	elif not detail.is_empty() and reason != "units_awaiting_orders" and not ERROR_TEXT.has(reason):
		text += " (%s)" % detail
	if not _joined:
		_connect.set_busy(false)
		_connect.set_status(text, true)
		_close_connection()
		return
	if reason == "units_awaiting_orders" and net != null:
		net.send_request("get_snapshot", {})  # resync the idle list the gate is based on
	_report(text, true)
	if meta.get("kind", "") == "ready":
		_agenda.reset_ready()


# ---------------------------------------------------------------- demonstrations

func _start_demo(kind: String) -> void:
	_close_connection()
	var parsed: Dictionary
	_catalog = null
	if kind == "servidor":
		var catalog := CatalogView.load_fixture(CATALOG_PATH)
		if catalog["ok"]:
			_catalog = catalog["catalog"]
		var file := FileAccess.open(SERVER_VIEW_PATH, FileAccess.READ)
		var data: Variant = JSON.parse_string(file.get_as_text()) if file != null else null
		if typeof(data) != TYPE_DICTIONARY:
			_connect.set_status("Não foi possível abrir a fixture da visão do servidor.", true)
			return
		if "--no-city" in OS.get_cmdline_user_args():
			data.erase("cities")
			data["home_tile"] = 178
			data["idle_units"] = []
		_last_payload = data
		parsed = ServerView.from_payload(data, _catalog)
	else:
		_last_payload = {}
		parsed = WorldView.load_fixture(FIXTURE_PATH)
	if not parsed["ok"]:
		_connect.set_status("Falha ao carregar a demonstração: %s" % parsed["error"], true)
		return
	world = parsed["view"]
	mode = Mode.DEMO
	_overrides.clear()
	_overrides_turn = world.turn
	_enter_game(true)


# ---------------------------------------------------------------- populate

func _populate(first: bool) -> void:
	if world.live:
		_header.text = "Turno %d · %s · mundo %d" % [world.turn, world.civilization.get("name", "?"), world.world_id]
	else:
		_header.text = "Turno %d · %s · %s" % [world.turn, world.era, world.civilization.get("name", "?")]
	for child in _resource_box.get_children():
		child.queue_free()
	for chip in _chips():
		var icon := TextureRect.new()
		icon.texture = load("res://assets/icons/%s.svg" % chip["icon"])
		icon.custom_minimum_size = Vector2(28, 28)
		icon.expand_mode = TextureRect.EXPAND_IGNORE_SIZE
		icon.stretch_mode = TextureRect.STRETCH_KEEP_ASPECT_CENTERED
		icon.tooltip_text = chip["label"]
		var label := Label.new()
		label.text = str(chip["value"])
		label.add_theme_font_size_override("font_size", 20)
		_resource_box.add_child(icon)
		_resource_box.add_child(label)
	_agenda.build(world.agenda, world.turn)
	_agenda.set_idle_units(_idle_entries())
	_refresh_capital_card()
	if mode == Mode.LIVE:
		_agenda.set_status("")
	else:
		_agenda.set_status("Demonstração (arquivo de exemplo): sem conexão com o servidor.")
	_map.set_view(world, first)
	_map.set_selected_unit(_selected_unit if not world.unit_by_id(_selected_unit).is_empty() else -1)
	_refresh_tech()
	if _selected_unit >= 0 and world.unit_by_id(_selected_unit).is_empty():
		_selected_unit = -1
		_unit_panel.clear()
	_refresh_panels()


func _chips() -> Array:
	if not world.header_chips.is_empty():
		return world.header_chips
	var chips: Array = []
	for resource in ["comida", "producao", "metal", "luxo"]:
		chips.append({"icon": resource, "value": int(world.resources.get(resource, 0)), "label": resource.capitalize()})
	return chips


func _refresh_tech() -> void:
	var available: bool = _catalog != null and world.live
	_nav_buttons["pesquisa"].visible = available
	if available:
		_tech.build(_catalog, world.research)
	elif _screens["pesquisa"].visible:
		show_screen("pauta")


func _idle_entries() -> Array:
	var entries: Array = []
	for unit_id in world.idle_units:
		var unit: Dictionary = world.unit_by_id(unit_id)
		if unit.is_empty():
			continue
		entries.append({"id": unit_id, "label": "%s · (%d, %d)" % [unit["name"], unit["cell"].x, unit["cell"].y]})
	return entries


func _refresh_units() -> void:
	_agenda.set_idle_units(_idle_entries())
	_refresh_capital_card()
	_map.set_view(world, false)
	_map.set_selected_unit(_selected_unit)
	_refresh_panels()


func _refresh_capital_card() -> void:
	var cell: Vector2i = world.home_cell if world.needs_capital() else Vector2i(-1, -1)
	_agenda.set_capital_card(cell, world.is_founding(world.home_cell))


## Redraws whichever bottom card is open (unit, city or starting tile) from the current view.
func _refresh_panels() -> void:
	if _selected_unit >= 0 and not world.unit_by_id(_selected_unit).is_empty():
		var unit: Dictionary = world.unit_by_id(_selected_unit)
		_unit_panel.show_unit(unit, world.turn, world.tile_at(unit["cell"]))
	if _selected_city >= 0:
		var city: Dictionary = world.city_by_id(_selected_city)
		if city.is_empty() or not city["own"]:
			_clear_city_selection()
		else:
			_city_panel.show_city(city, _catalog, world.research)
	elif _founding_selected:
		if world.needs_capital():
			_city_panel.show_founding(world.home_cell, world.is_founding(world.home_cell))
		else:
			_clear_city_selection()


func _report(text: String, is_error: bool) -> void:
	if mode == Mode.CONNECT:
		_connect.set_status(text, is_error)
	else:
		_agenda.set_status(text)
		_tile_info.text = text


# ---------------------------------------------------------------- map and units

func _on_cell_selected(cell: Vector2i) -> void:
	if cell.x < 0:
		_tile_info.text = "Fora do mapa"
		return
	var tile: Dictionary = world.tile_at(cell)
	if tile["fog"] == "unknown":
		_tile_info.text = "(%d, %d) · território desconhecido" % [cell.x, cell.y]
		_clear_unit_selection()
		return
	var parts: Array[String] = ["(%d, %d)" % [cell.x, cell.y], BIOME_NAMES.get(tile["biome"], tile["biome"])]
	if tile.has("city"):
		parts.append(tile["city"]["name"])
	if tile.has("resource"):
		parts.append("recurso: %s" % tile["resource"])
	parts.append("visível" if tile["fog"] == "visible" else "lembrado")
	_tile_info.text = " · ".join(parts)
	# Tapping the same tile again cycles through its stack: own city (or the starting tile), then units.
	var entries: Array = []
	var city: Dictionary = world.city_at(cell)
	if not city.is_empty() and city["own"]:
		entries.append({"city": city["id"]})
	elif city.is_empty() and world.needs_capital() and cell == world.home_cell:
		entries.append({"home": true})
	for unit in world.units_at(cell):
		entries.append({"unit": unit["id"]})
	if entries.is_empty():
		_clear_unit_selection()
		_clear_city_selection()
		return
	var pick := 0
	for i in entries.size():
		var entry: Dictionary = entries[i]
		if (entry.has("unit") and entry["unit"] == _selected_unit) or (entry.has("city") and entry["city"] == _selected_city) or (entry.has("home") and _founding_selected):
			pick = (i + 1) % entries.size()
	var chosen: Dictionary = entries[pick]
	if chosen.has("unit"):
		_select_unit(chosen["unit"])
	else:
		_clear_unit_selection()
		_select_city(chosen["city"] if chosen.has("city") else -1)


func _select_city(city_id: int) -> void:
	_set_move_mode(false)
	_selected_city = city_id
	_founding_selected = city_id < 0
	_refresh_panels()


func _clear_city_selection() -> void:
	_selected_city = -1
	_founding_selected = false
	_city_panel.clear()


func _clear_unit_selection() -> void:
	_selected_unit = -1
	_unit_panel.clear()
	_map.set_selected_unit(-1)
	_set_move_mode(false)


func _select_unit(unit_id: int) -> void:
	var unit: Dictionary = world.unit_by_id(unit_id)
	if unit.is_empty():
		return
	_set_move_mode(false)
	_clear_city_selection()
	_selected_unit = unit_id
	_unit_panel.show_unit(unit, world.turn, world.tile_at(unit["cell"]))
	_map.set_selected_unit(unit_id)


func _on_unit_focus_requested(unit_id: int) -> void:
	var unit: Dictionary = world.unit_by_id(unit_id)
	if unit.is_empty():
		return
	show_screen("mapa")
	_map.focus_cell(unit["cell"])
	_map.selected = unit["cell"]
	_select_unit(unit_id)


func _on_capital_focus_requested(cell: Vector2i) -> void:
	show_screen("mapa")
	_map.focus_cell(cell)
	_map.selected = cell
	_clear_unit_selection()
	_select_city(-1)
	_tile_info.text = "(%d, %d) · ponto inicial" % [cell.x, cell.y]


func _set_move_mode(on: bool) -> void:
	_move_mode = on
	_map.set_target_mode(on)
	_unit_panel.set_move_mode(on)


func _on_unit_action(action: String) -> void:
	if _selected_unit < 0:
		return
	var unit_id := _selected_unit
	match action:
		"move":
			_set_move_mode(not _move_mode)
		"explore":
			_issue_order(unit_id, "Explore", Vector2i(-1, -1))
		"fortify":
			_issue_order(unit_id, "Fortify", Vector2i(-1, -1))
		"found":
			var unit: Dictionary = world.unit_by_id(unit_id)
			_issue_found(Protocol.settler_city_id(unit_id), unit["cell"])
		"skip":
			_issue({"kind": "skip", "unit_id": unit_id}, Protocol.command_skip_unit(unit_id))


func _on_target_chosen(cell: Vector2i) -> void:
	var unit_id := _selected_unit
	_set_move_mode(false)
	if unit_id < 0:
		return
	if world.tile_at(cell)["fog"] == "unknown":
		_report("Destino desconhecido: escolha um tile já explorado.", true)
		return
	_issue_order(unit_id, "MoveTo", cell)


func _issue_order(unit_id: int, kind: String, target: Vector2i) -> void:
	var index := Hex.index_of_cell(target, world.map_width) if kind == "MoveTo" else -1
	var order := Protocol.order_wire(kind, index)
	_issue({"kind": "order", "unit_id": unit_id, "order_kind": kind, "order_target": target}, Protocol.command_set_unit_order(unit_id, order))


func _on_found_capital_requested() -> void:
	if world == null or not world.needs_capital() or world.is_founding(world.home_cell):
		return
	_issue_found(world.civ_id, world.home_cell)


func _issue_found(city_id: int, cell: Vector2i) -> void:
	var index := Hex.index_of_cell(cell, world.map_width)
	_issue({"kind": "found", "city_id": city_id, "cell": cell}, Protocol.command_found_city(city_id, index))


func _on_queue_requested(unit_type: String) -> void:
	if _selected_city < 0:
		return
	_issue({"kind": "queue", "city_id": _selected_city, "unit_type": unit_type}, Protocol.command_queue_unit(_selected_city, unit_type))


## Sends a unit command. Live: the order only takes effect locally once the server accepts it.
## Demonstration: applied locally at once, since there is no server to ask.
func _issue(meta: Dictionary, payload: Dictionary) -> void:
	if mode == Mode.LIVE:
		var request_id: String = net.send_request("submit_command", payload) if net != null else ""
		if request_id.is_empty():
			_report("Sem conexão com o servidor: a ordem não foi enviada.", true)
			return
		_inflight[request_id] = meta
	elif mode == Mode.DEMO and world.live:
		_overrides.append(meta)
		_apply_override(meta)
		_refresh_units()
		_report("Demonstração: a ordem vale só neste cliente.", false)


func _apply_override(meta: Dictionary) -> void:
	match meta["kind"]:
		"skip":
			world.apply_local_skip(meta["unit_id"])
		"found":
			world.apply_local_found(meta["cell"])
		"queue":
			world.apply_local_queue(meta["city_id"], meta["unit_type"])
		_:
			world.apply_local_order(meta["unit_id"], meta["order_kind"], meta["order_target"])


# ---------------------------------------------------------------- ready

func _on_ready_pressed(choices: Dictionary) -> void:
	if world != null and world.ready_blocked():
		_agenda.set_status("%s: dê uma ordem ou pule." % WorldView.gate_text(world.awaiting_count()))
		return
	if mode == Mode.LIVE and net != null and net.is_open():
		for card in world.agenda:
			if card.has("event_id") and choices.has(card["id"]):
				net.send_request("submit_command", Protocol.command_respond_to_event(card["event_id"], choices[card["id"]]))
		var request_id: String = net.send_request("ready", {})
		_inflight[request_id] = {"kind": "ready"}
		_agenda.set_status("Pronto enviado; aguardando o servidor.")
	else:
		_agenda.mark_ready(false)
		_agenda.set_status("Demonstração: nada foi enviado. Escolhas ficaram apenas neste rascunho local.")


func _flat(color: Color, margin: int) -> StyleBoxFlat:
	var style := StyleBoxFlat.new()
	style.bg_color = color
	style.set_content_margin_all(margin)
	return style

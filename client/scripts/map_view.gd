extends Control
## Map screen: draws the known tiles, drag to pan, wheel/pinch to zoom, tap to select.

const Hex := preload("res://scripts/hex.gd")

signal cell_selected(cell: Vector2i)
## Emitted instead of `cell_selected` while in target mode (choosing a destination).
signal target_chosen(cell: Vector2i)

const UNIT_DISC := Color("f4f0da")
const ROLE_ICONS := {
	"exploration": "unit-scout", "defense": "unit-militia", "attack": "unit-raider",
	"settler": "unit-settler", "worker": "unit-worker", "trade": "unit-caravan",
}

const RADIUS := 64.0
const ZOOM_MIN := 0.3
const ZOOM_MAX := 1.4
const TAP_SLOP := 14.0
const BG_COLOR := Color("17212b")
const CITY_COLOR := Color("f4f0da")
const SELECT_COLOR := Color("e69f00")

var view: RefCounted
var catalog: RefCounted  # CatalogView, optional: names for improvement markers
var camera := Vector2.ZERO
var zoom := 0.6
var selected := Vector2i(-1, -1)
var selected_unit := -1
var target_mode := false

var _units_by_cell: Dictionary = {}
var _unit_icons: Dictionary = {}

var _tiles: Dictionary = {}
var _overlays: Dictionary = {}
var _icons: Dictionary = {}
var _city: Texture2D
var _dragging := false
var _press_pos := Vector2.ZERO
var _camera_at_press := Vector2.ZERO
var _touches: Dictionary = {}
var _pinch_last := 0.0


func _ready() -> void:
	clip_contents = true
	for biome in ["oceano", "costa", "planicie", "floresta", "selva", "savana", "deserto", "estepe", "tundra", "pantano", "montanha", "geleira"]:
		_tiles[biome] = load("res://assets/tiles/%s-1.svg" % biome)
	for edge in Hex.EDGES:
		_overlays["fronteira-" + edge] = load("res://assets/tiles/overlays/fronteira-%s.svg" % edge)
		_overlays["rio-" + edge] = load("res://assets/tiles/overlays/rio-%s.svg" % edge)
	_overlays["remembered"] = load("res://assets/tiles/overlays/nevoa-lembrado.svg")
	_overlays["unknown"] = load("res://assets/tiles/overlays/nevoa-desconhecido.svg")
	for resource in ["comida", "producao", "metal", "luxo"]:
		_icons[resource] = load("res://assets/icons/%s.svg" % resource)
	_city = load("res://assets/entities/cidade-media-normal.svg")


## Installs a new world view. `recenter` false keeps the camera (used when a turn refreshes the view).
func set_view(world_view: RefCounted, recenter: bool = true) -> void:
	view = world_view
	_units_by_cell.clear()
	for unit in view.units:
		if not _units_by_cell.has(unit["cell"]):
			_units_by_cell[unit["cell"]] = []
		_units_by_cell[unit["cell"]].append(unit)
	if recenter:
		camera = Hex.center(view.capital, RADIUS)
	queue_redraw()


## Centers the camera on a cell (used by the idle-unit queue).
func focus_cell(cell: Vector2i) -> void:
	if view == null or cell.x < 0:
		return
	camera = Hex.center(cell, RADIUS)
	_clamp_camera()
	queue_redraw()


func set_selected_unit(unit_id: int) -> void:
	selected_unit = unit_id
	queue_redraw()


## While true the next tap is reported with `target_chosen` instead of selecting.
func set_target_mode(on: bool) -> void:
	target_mode = on


func _unit_texture(unit: Dictionary) -> Texture2D:
	var type: String = unit["type"]
	if not _unit_icons.has(type):
		var path := "res://assets/entities/unidades/%s.svg" % type.replace("unit.", "unit-").replace("_", "-")
		if not ResourceLoader.exists(path):
			path = "res://assets/entities/unidades/%s.svg" % ROLE_ICONS.get(unit["role"], "unit-militia")
		_unit_icons[type] = load(path)
	return _unit_icons[type]


func _draw_units(screen: Vector2, cell: Vector2i, has_city: bool) -> void:
	if not _units_by_cell.has(cell):
		return
	var stack: Array = _units_by_cell[cell]
	var unit: Dictionary = stack[0]
	for candidate in stack:
		if candidate["id"] == selected_unit:
			unit = candidate
			break
		if candidate["own"] and not unit["own"]:
			unit = candidate
	var center := screen + (Vector2(RADIUS * 0.5, RADIUS * 0.4) * zoom if has_city else Vector2.ZERO)
	var radius := 23.0 * zoom
	var color: Color = view.civ_colors.get(unit["owner"], Color("b9c1c8"))
	draw_circle(center, radius + 3.0 * zoom, color)
	draw_circle(center, radius, UNIT_DISC)
	var icon_size := Vector2(36, 36) * zoom
	draw_texture_rect(_unit_texture(unit), Rect2(center - icon_size * 0.5, icon_size), false)
	if unit["own"] and view.idle_units.has(unit["id"]):
		draw_circle(center + Vector2(radius * 0.75, -radius * 0.75), 7.0 * zoom, SELECT_COLOR)
	if stack.size() > 1:
		var font := ThemeDB.fallback_font
		draw_string(font, center + Vector2(-radius, radius + 11.0 * zoom), "x%d" % stack.size(), HORIZONTAL_ALIGNMENT_LEFT, -1, int(16 * zoom + 4), CITY_COLOR)
	if unit["id"] == selected_unit:
		draw_arc(center, radius + 7.0 * zoom, 0.0, TAU, 24, SELECT_COLOR, 4.0)
		if unit["order"] == "MoveTo" and unit["order_target"].x >= 0:
			_draw_target_marker(unit["order_target"])


func _draw_target_marker(cell: Vector2i) -> void:
	var base := Hex.center(cell, RADIUS)
	var world_width: float = RADIUS * Hex.SQRT3 * view.map_width
	var best := base
	for wrap in [-1, 0, 1]:  # draw the copy nearest to the camera across the cylinder seam
		var candidate := base + Vector2(wrap * world_width, 0.0)
		if absf(candidate.x - camera.x) < absf(best.x - camera.x):
			best = candidate
	var screen := (best - camera) * zoom + size * 0.5
	draw_arc(screen, RADIUS * 0.5 * zoom, 0.0, TAU, 4, SELECT_COLOR, 3.0)


func _draw() -> void:
	draw_rect(Rect2(Vector2.ZERO, size), BG_COLOR)
	if view == null:
		return
	var world_width: float = RADIUS * Hex.SQRT3 * view.map_width
	var half := size * 0.5 / zoom
	var first_wrap := floori((camera.x - half.x - RADIUS * 2.0) / world_width)
	var last_wrap := floori((camera.x + half.x + RADIUS * 2.0) / world_width)
	var tile_size := Vector2(RADIUS * Hex.SQRT3, RADIUS * 2.0) * zoom
	for row in view.map_height:
		for col in view.map_width:
			var base := Hex.center(Vector2i(col, row), RADIUS)
			for wrap in range(first_wrap, last_wrap + 1):
				var screen := (base + Vector2(wrap * world_width, 0.0) - camera) * zoom + size * 0.5
				if screen.x < -tile_size.x or screen.x > size.x + tile_size.x or screen.y < -tile_size.y or screen.y > size.y + tile_size.y:
					continue
				_draw_tile(screen, tile_size, view.tiles[row][col], Vector2i(col, row))


func _draw_tile(screen: Vector2, tile_size: Vector2, tile: Dictionary, cell: Vector2i) -> void:
	var rect := Rect2(screen - tile_size * 0.5, tile_size)
	var fog: String = tile["fog"]
	if fog == "unknown":
		draw_texture_rect(_overlays["unknown"], rect, false)
		return
	draw_texture_rect(_tiles[tile["biome"]], rect, false)
	for edge in tile.get("rivers", []):
		draw_texture_rect(_overlays["rio-" + edge], rect, false)
	for edge in tile.get("borders", []):
		draw_texture_rect(_overlays["fronteira-" + edge], rect, false)
	if tile.has("resource"):
		var icon_size := Vector2(31, 31) * zoom
		draw_texture_rect(_icons[tile["resource"]], Rect2(screen - icon_size * 0.5, icon_size), false)
	_draw_improvement(screen, tile)
	if tile.has("city"):
		var city_size := Vector2(46, 46) * zoom
		draw_texture_rect(_city, Rect2(screen - city_size * 0.5, city_size), false)
	if fog == "remembered":
		draw_texture_rect(_overlays["remembered"], rect, false)
	_draw_units(screen, cell, tile.has("city"))
	if cell == selected:
		draw_arc(screen, RADIUS * 0.84 * zoom, 0.0, TAU, 24, SELECT_COLOR, 4.0)


## Improvements are catalog ids in the projection. A small blue marker keeps them distinct from
## resources; an amber hammer/progress label marks an unfinished work without inventing a rate.
## First letter of the catalog name (or of the id's last segment without a catalog).
static func improvement_initial(id: String, catalog: RefCounted = null) -> String:
	var label: String = catalog.improvement_name(id) if catalog != null else id.get_slice(".", id.get_slice_count(".") - 1)
	return label.left(1).to_upper()


func _draw_improvement(screen: Vector2, tile: Dictionary) -> void:
	if tile.has("improvement"):
		var marker := Rect2(screen + Vector2(-15, 20) * zoom, Vector2(30, 14) * zoom)
		draw_rect(marker, Color("56b4e9"), true)
		draw_rect(marker, Color("17212b"), false, 2.0)
		var id := String(tile["improvement"])
		var text := improvement_initial(id, catalog)
		draw_string(ThemeDB.fallback_font, marker.position + Vector2(2, marker.size.y - 2), text, HORIZONTAL_ALIGNMENT_LEFT, marker.size.x - 2, int(10 * zoom + 4), Color("17212b"))
	if tile.has("build_progress"):
		var progress: Variant = tile["build_progress"]
		var hammer := screen + Vector2(-19, 31) * zoom
		draw_line(hammer + Vector2(3, 14) * zoom, hammer + Vector2(13, 3) * zoom, SELECT_COLOR, 5.0 * zoom, true)
		draw_line(hammer + Vector2(9, 1) * zoom, hammer + Vector2(16, 8) * zoom, SELECT_COLOR, 5.0 * zoom, true)
		draw_line(hammer + Vector2(8, 3) * zoom, hammer + Vector2(14, -3) * zoom, SELECT_COLOR, 5.0 * zoom, true)
		var label := ""
		if typeof(progress) == TYPE_DICTIONARY:
			var current: Variant = progress.get("progress")
			var required: Variant = progress.get("required")
			if current != null and required != null:
				label = "%s/%s" % [str(current), str(required)]
		if not label.is_empty():
			draw_string(ThemeDB.fallback_font, screen + Vector2(-20, 47) * zoom, label, HORIZONTAL_ALIGNMENT_LEFT, -1, int(12 * zoom + 5), SELECT_COLOR)


func _gui_input(event: InputEvent) -> void:
	if event is InputEventMouseButton:
		var mb := event as InputEventMouseButton
		if mb.button_index == MOUSE_BUTTON_WHEEL_UP and mb.pressed:
			_set_zoom(zoom * 1.12)
		elif mb.button_index == MOUSE_BUTTON_WHEEL_DOWN and mb.pressed:
			_set_zoom(zoom / 1.12)
		elif mb.button_index == MOUSE_BUTTON_LEFT:
			if mb.pressed:
				_dragging = true
				_press_pos = mb.position
				_camera_at_press = camera
			else:
				if _dragging and mb.position.distance_to(_press_pos) < TAP_SLOP and _touches.size() < 2:
					select_at(mb.position)
				_dragging = false
	elif event is InputEventMouseMotion and _dragging and _touches.size() < 2:
		camera = _camera_at_press - ((event as InputEventMouseMotion).position - _press_pos) / zoom
		_clamp_camera()
		queue_redraw()
	elif event is InputEventScreenTouch:
		var st := event as InputEventScreenTouch
		if st.pressed:
			_touches[st.index] = st.position
		else:
			_touches.erase(st.index)
			_pinch_last = 0.0
	elif event is InputEventScreenDrag:
		var sd := event as InputEventScreenDrag
		_touches[sd.index] = sd.position
		if _touches.size() >= 2:
			var points: Array = _touches.values()
			var d: float = points[0].distance_to(points[1])
			if _pinch_last > 0.0:
				_set_zoom(zoom * d / _pinch_last)
			_pinch_last = d


func _set_zoom(value: float) -> void:
	zoom = clampf(value, ZOOM_MIN, ZOOM_MAX)
	_clamp_camera()
	queue_redraw()


func _clamp_camera() -> void:
	if view == null:
		return
	var world_width: float = RADIUS * Hex.SQRT3 * view.map_width
	camera.x = fposmod(camera.x, world_width)
	camera.y = clampf(camera.y, 0.0, RADIUS * 1.5 * (view.map_height - 1))


## Converts a screen position (control-local) to the cell under it, or (-1,-1).
func cell_at(screen_position: Vector2) -> Vector2i:
	if view == null:
		return Vector2i(-1, -1)
	var world := (screen_position - size * 0.5) / zoom + camera
	return Hex.pixel_to_cell(world, RADIUS, view.map_width, view.map_height)


func select_at(screen_position: Vector2) -> void:
	select_cell(cell_at(screen_position))


func select_cell(cell: Vector2i) -> void:
	if target_mode:
		if cell.x >= 0:
			target_chosen.emit(cell)
		return
	selected = cell
	queue_redraw()
	cell_selected.emit(cell)

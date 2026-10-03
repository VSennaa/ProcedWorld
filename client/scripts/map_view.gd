extends Control
## Map screen: draws the known tiles, drag to pan, wheel/pinch to zoom, tap to select.

const Hex := preload("res://scripts/hex.gd")

signal cell_selected(cell: Vector2i)

const RADIUS := 64.0
const ZOOM_MIN := 0.3
const ZOOM_MAX := 1.4
const TAP_SLOP := 14.0
const BG_COLOR := Color("17212b")
const CITY_COLOR := Color("f4f0da")
const SELECT_COLOR := Color("e69f00")

var view: RefCounted
var camera := Vector2.ZERO
var zoom := 0.6
var selected := Vector2i(-1, -1)

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


func set_view(world_view: RefCounted) -> void:
	view = world_view
	camera = Hex.center(view.capital, RADIUS)
	queue_redraw()


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
	if tile.has("city"):
		var city_size := Vector2(46, 46) * zoom
		draw_texture_rect(_city, Rect2(screen - city_size * 0.5, city_size), false)
	if fog == "remembered":
		draw_texture_rect(_overlays["remembered"], rect, false)
	if cell == selected:
		draw_arc(screen, RADIUS * 0.84 * zoom, 0.0, TAU, 24, SELECT_COLOR, 4.0)


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
	selected = cell
	queue_redraw()
	cell_selected.emit(cell)

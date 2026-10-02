extends Node2D

const HexGeometry = preload("res://scripts/hex_geometry.gd")
const MapGenerator = preload("res://scripts/map_generator.gd")
const MAP_WIDTH := 40
const MAP_HEIGHT := 40
const SEED := 20261001
const RADIUS := 64.0
const ZOOM_MIN := 0.35
const ZOOM_MAX := 1.35

var map_data: Dictionary
var textures: Dictionary = {}
var overlays: Dictionary = {}
var icons: Dictionary = {}
var camera := Vector2.ZERO
var zoom := 0.62
var selected := Vector2i(-1, -1)
var dragging := false
var drag_start := Vector2.ZERO
var camera_start := Vector2.ZERO
var last_pinch_distance := 0.0
var touch_positions: Dictionary = {}
var info_label: Label

func _ready() -> void:
	map_data = MapGenerator.generate(MAP_WIDTH, MAP_HEIGHT, SEED)
	for biome in MapGenerator.BIOMES:
		textures[biome] = load("res://assets/tiles/%s-1.svg" % biome)
	for direction in ["ne", "e", "se", "sw", "w", "nw"]:
		overlays["rio-" + direction] = load("res://assets/tiles/overlays/rio-%s.svg" % direction)
		overlays["fronteira-" + direction] = load("res://assets/tiles/overlays/fronteira-%s.svg" % direction)
	overlays["remembered"] = load("res://assets/tiles/overlays/nevoa-lembrado.svg")
	overlays["unknown"] = load("res://assets/tiles/overlays/nevoa-desconhecido.svg")
	for resource in ["comida", "metal", "luxo", "producao"]:
		icons[resource] = load("res://assets/icons/%s.svg" % resource)
	info_label = Label.new()
	info_label.position = Vector2(16, 16)
	info_label.add_theme_font_size_override("font_size", 20)
	info_label.add_theme_color_override("font_color", Color("f4f0da"))
	info_label.add_theme_color_override("font_shadow_color", Color("18212d"))
	info_label.add_theme_constant_override("shadow_offset_x", 2)
	info_label.add_theme_constant_override("shadow_offset_y", 2)
	info_label.text = "Mapa 40×40 · seed %d · 8 civilizações\nArraste para mover · roda/pinça para zoom\nToque em um hexágono" % SEED
	add_child(info_label)
	camera = HexGeometry.cell_center(map_data.capitals[0], RADIUS)
	queue_redraw()

func _draw() -> void:
	draw_rect(Rect2(Vector2.ZERO, get_viewport_rect().size), Color("17212b"))
	var viewport := get_viewport_rect().size
	var world_width := RADIUS * HexGeometry.SQRT_3 * MAP_WIDTH
	var left_world := camera.x - viewport.x * 0.5 / zoom - RADIUS * 2.0
	var right_world := camera.x + viewport.x * 0.5 / zoom + RADIUS * 2.0
	for r in MAP_HEIGHT:
		for q in MAP_WIDTH:
			var base := HexGeometry.cell_center(Vector2i(q, r), RADIUS)
			for wrap in range(floori(left_world / world_width) - 1, floori(right_world / world_width) + 2):
				var screen := (base + Vector2(wrap * world_width, 0) - camera) * zoom + viewport * 0.5
				if screen.x < -RADIUS * zoom or screen.x > viewport.x + RADIUS * zoom or screen.y < -RADIUS * zoom or screen.y > viewport.y + RADIUS * zoom:
					continue
				draw_tile(screen, map_data.tiles[r][q], Vector2i(q, r))

func draw_tile(screen: Vector2, tile: Dictionary, cell: Vector2i) -> void:
	var size := Vector2(RADIUS * HexGeometry.SQRT_3, RADIUS * 2.0) * zoom
	draw_texture_rect(textures[tile.biome], Rect2(screen - size * 0.5, size), false)
	if tile.fog != "unknown":
		for direction in tile.rivers:
			draw_texture_rect(overlays["rio-" + direction], Rect2(screen - size * 0.5, size), false)
		for direction in tile.borders:
			draw_texture_rect(overlays["fronteira-" + direction], Rect2(screen - size * 0.5, size), false)
		if tile.resource != "":
			var icon_size := Vector2(31, 31) * zoom
			draw_texture_rect(icons[tile.resource], Rect2(screen - icon_size * 0.5, icon_size), false)
	if tile.fog != "visible":
		draw_texture_rect(overlays[tile.fog], Rect2(screen - size * 0.5, size), false)
	if cell == map_data.capitals[0] and tile.fog == "visible":
		draw_circle(screen, 7.0 * zoom, Color("f4f0da"))
	if selected == cell:
		draw_arc(screen, RADIUS * 0.84 * zoom, 0.0, TAU, 24, Color("e69f00"), 4.0)

func _unhandled_input(event: InputEvent) -> void:
	if event is InputEventMouseButton:
		if event.button_index == MOUSE_BUTTON_WHEEL_UP:
			set_zoom(zoom * 1.12)
		elif event.button_index == MOUSE_BUTTON_WHEEL_DOWN:
			set_zoom(zoom / 1.12)
		elif event.button_index == MOUSE_BUTTON_LEFT:
			if event.pressed:
				dragging = true
				drag_start = event.position
				camera_start = camera
			else:
				if dragging and event.position.distance_to(drag_start) < 12.0:
					select_at(event.position)
				dragging = false
	elif event is InputEventMouseMotion and dragging:
		camera = camera_start - (event.position - drag_start) / zoom
		queue_redraw()
	elif event is InputEventScreenTouch:
		if event.pressed:
			touch_positions[event.index] = event.position
			dragging = true
			drag_start = event.position
			camera_start = camera
		else:
			touch_positions.erase(event.index)
			if event.position.distance_to(drag_start) < 16.0:
				select_at(event.position)
			dragging = false
			last_pinch_distance = 0.0
	elif event is InputEventScreenDrag:
		touch_positions[event.index] = event.position
		if touch_positions.size() >= 2:
			var points := touch_positions.values()
			var pinch_distance: float = points[0].distance_to(points[1])
			if last_pinch_distance > 0.0:
				set_zoom(zoom * pinch_distance / last_pinch_distance)
			last_pinch_distance = pinch_distance
		else:
			camera = camera_start - (event.position - drag_start) / zoom
		queue_redraw()

func set_zoom(value: float) -> void:
	zoom = clampf(value, ZOOM_MIN, ZOOM_MAX)
	queue_redraw()

func select_at(screen_position: Vector2) -> void:
	var world_position := (screen_position - get_viewport_rect().size * 0.5) / zoom + camera
	selected = HexGeometry.pixel_to_cell(world_position, RADIUS, MAP_WIDTH, MAP_HEIGHT)
	var tile: Dictionary = map_data.tiles[selected.y][selected.x]
	info_label.text = "(%d, %d) · %s · névoa: %s\nArraste para mover · roda/pinça para zoom" % [selected.x, selected.y, tile.biome.capitalize(), tile.fog]
	queue_redraw()

extends SceneTree

const HexGeometry = preload("res://scripts/hex_geometry.gd")
const MapGenerator = preload("res://scripts/map_generator.gd")
var failures := 0

func expect(condition: bool, message: String) -> void:
	if not condition:
		failures += 1
		print("FAIL: ", message)

func _init() -> void:
	var width := 40
	var height := 40
	expect(MapGenerator.generate(width, height, 20261001) == MapGenerator.generate(width, height, 20261001), "geração inteira deve repetir para a mesma seed")
	var seam_neighbors := HexGeometry.neighbors(Vector2i(0, 12), width, height)
	expect(seam_neighbors.has(Vector2i(39, 12)), "vizinho oeste deve cruzar a costura")
	expect(HexGeometry.neighbors(Vector2i(4, 0), width, height).size() == 4, "polo norte deve omitir dois vizinhos")
	expect(HexGeometry.neighbors(Vector2i(4, height - 1), width, height).size() == 4, "polo sul deve omitir dois vizinhos")
	expect(HexGeometry.wrap_distance(Vector2i(0, 10), Vector2i(39, 10), width) == 1, "distância deve usar wrap horizontal")
	expect(HexGeometry.wrap_distance(Vector2i(8, 7), Vector2i(8, 7), width) == 0, "distância própria deve ser zero")
	for cell in [Vector2i(0, 0), Vector2i(39, 17), Vector2i(12, 31)]:
		var point := HexGeometry.cell_center(cell, 64.0)
		expect(HexGeometry.pixel_to_cell(point, 64.0, width, height) == cell, "pixel→hex no centro %s" % cell)
	var seam_point := HexGeometry.cell_center(Vector2i(-1, 9), 64.0)
	expect(HexGeometry.pixel_to_cell(seam_point, 64.0, width, height) == Vector2i(39, 9), "pixel→hex deve normalizar a costura")
	if failures == 0:
		print("ALL TESTS PASSED")
		quit(0)
	else:
		print("%d TESTS FAILED" % failures)
		quit(1)

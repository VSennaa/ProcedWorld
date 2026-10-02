class_name MapGenerator
extends RefCounted

const HexGeometry = preload("res://scripts/hex_geometry.gd")
const BIOMES := ["oceano", "costa", "planicie", "floresta", "deserto", "montanha", "geleira"]
const RIVER_DIRECTIONS := ["ne", "e", "se", "sw", "w", "nw"]
const OPPOSITE_DIRECTIONS := ["sw", "w", "nw", "ne", "e", "se"]
const CIVILIZATION_COUNT := 8
const LAND_PERCENT := 35

static func generate(width: int, height: int, seed: int) -> Dictionary:
	# Presentation-only port of the deterministic-map spike. Every pass has fixed
	# row-major order and derives its values from the explicit seed.
	var elevations: Array = []
	var ranked: Array = []
	for r in height:
		var row: Array = []
		for q in width:
			var elevation := smooth_noise(q, r, width, height, seed)
			row.append(elevation)
			ranked.append(elevation * 10000 + r * width + q)
		elevations.append(row)
	ranked.sort()
	var land_cutoff: int = ranked[ranked.size() - (width * height * LAND_PERCENT / 100)]
	var sea_level: int = land_cutoff / 10000

	var tiles: Array = []
	for r in height:
		var row: Array = []
		for q in width:
			var elevation: int = elevations[r][q]
			var latitude: int = int(abs(r * 2 - (height - 1)) * 100 / (height - 1))
			var is_land := elevation * 10000 + r * width + q >= land_cutoff
			var biome := biome_for(elevation, sea_level, latitude, q, r, seed, is_land)
			row.append({"biome": biome, "elevation": elevation, "land": is_land, "rivers": [], "borders": [], "owner": -1, "fog": "unknown", "resource": resource_for(biome, q, r, seed)})
		tiles.append(row)
	add_rivers(tiles, width, height, seed)
	var capitals := choose_capitals(tiles, width, height, seed)
	assign_territories(tiles, capitals, width, height)
	assign_fog(tiles, capitals[0], width, height)
	return {"tiles": tiles, "capitals": capitals, "sea_level": sea_level}

static func smooth_noise(q: int, r: int, width: int, height: int, seed: int) -> int:
	var total := 0
	var weight := 0
	for sample in [[0, 0, 8], [-1, 0, 4], [1, 0, 4], [0, -1, 4], [0, 1, 4], [-1, -1, 2], [1, 1, 2]]:
		var sample_q: int = posmod(q / 5 + sample[0], maxi(1, width / 5))
		var sample_r: int = clampi(r / 5 + sample[1], 0, maxi(0, height / 5 - 1))
		total += cell_noise(sample_q, sample_r, seed ^ 0x5f3759df) * sample[2]
		weight += sample[2]
	var detail := cell_noise(q, r, seed) / 5
	return total / weight + detail

static func biome_for(elevation: int, sea_level: int, latitude: int, q: int, r: int, seed: int, is_land: bool) -> String:
	if not is_land:
		return "costa" if elevation >= sea_level - 3 else "oceano"
	if latitude >= 82:
		return "geleira"
	if elevation >= sea_level + 20:
		return "montanha"
	var moisture := cell_noise(q / 3, r / 3, seed ^ 0x4a39b70d)
	if latitude >= 62:
		return "planicie"
	if moisture >= 58:
		return "floresta"
	if moisture <= 25:
		return "deserto"
	return "planicie"

static func resource_for(biome: String, q: int, r: int, seed: int) -> String:
	if biome == "oceano" or biome == "costa":
		return ""
	var roll := cell_noise(q, r, seed ^ 0x3c6ef372) % 17
	if roll == 0:
		return "metal"
	if roll == 1:
		return "luxo"
	if roll == 2:
		return "producao"
	if roll == 3:
		return "comida"
	return ""

static func add_rivers(tiles: Array, width: int, height: int, seed: int) -> void:
	for r in height:
		for q in width:
			var tile: Dictionary = tiles[r][q]
			if not tile.land or tile.biome == "geleira" or tile.elevation < 58:
				continue
			if cell_noise(q, r, seed ^ 0x9e3779b9) % 23 != 0:
				continue
			var current := Vector2i(q, r)
			for step in 7:
				var next := lowest_neighbor(current, tiles, width, height)
				if next == current:
					break
				var direction := direction_to(current, next, width, height)
				if direction < 0:
					break
				var current_tile: Dictionary = tiles[current.y][current.x]
				var next_tile: Dictionary = tiles[next.y][next.x]
				if not current_tile.rivers.has(RIVER_DIRECTIONS[direction]):
					current_tile.rivers.append(RIVER_DIRECTIONS[direction])
					next_tile.rivers.append(OPPOSITE_DIRECTIONS[direction])
				if not next_tile.land:
					break
				current = next

static func lowest_neighbor(cell: Vector2i, tiles: Array, width: int, height: int) -> Vector2i:
	var result := cell
	var best: int = tiles[cell.y][cell.x].elevation
	for neighbor in HexGeometry.neighbors(cell, width, height):
		var elevation: int = tiles[neighbor.y][neighbor.x].elevation
		if elevation < best:
			best = elevation
			result = neighbor
	return result

static func direction_to(from: Vector2i, to: Vector2i, width: int, height: int) -> int:
	return HexGeometry.neighbors(from, width, height).find(to)

static func choose_capitals(tiles: Array, width: int, height: int, seed: int) -> Array:
	var candidates: Array = []
	for r in height:
		for q in width:
			var tile: Dictionary = tiles[r][q]
			if tile.land and tile.biome != "geleira" and tile.biome != "montanha":
				candidates.append({"cell": Vector2i(q, r), "score": cell_noise(q, r, seed ^ 0xbb67ae85)})
	candidates.sort_custom(func(a: Dictionary, b: Dictionary) -> bool:
		if a.score == b.score:
			return a.cell.y * width + a.cell.x < b.cell.y * width + b.cell.x
		return a.score > b.score)
	var capitals: Array = []
	for candidate in candidates:
		var far_enough := true
		for capital in capitals:
			if HexGeometry.wrap_distance(candidate.cell, capital, width) < 8:
				far_enough = false
				break
		if far_enough:
			capitals.append(candidate.cell)
			if capitals.size() == CIVILIZATION_COUNT:
				return capitals
	return capitals

static func assign_territories(tiles: Array, capitals: Array, width: int, height: int) -> void:
	for r in height:
		for q in width:
			var tile: Dictionary = tiles[r][q]
			if not tile.land:
				continue
			var cell := Vector2i(q, r)
			var owner := -1
			var distance := 999
			for civilization in capitals.size():
				var candidate_distance := HexGeometry.wrap_distance(cell, capitals[civilization], width)
				if candidate_distance < distance:
					distance = candidate_distance
					owner = civilization
			if distance <= 7:
				tile.owner = owner
	for r in height:
		for q in width:
			var tile: Dictionary = tiles[r][q]
			if tile.owner < 0:
				continue
			var neighbors := HexGeometry.neighbors(Vector2i(q, r), width, height)
			for index in neighbors.size():
				var neighbor: Vector2i = neighbors[index]
				if tiles[neighbor.y][neighbor.x].owner != tile.owner:
					tile.borders.append(RIVER_DIRECTIONS[index])

static func assign_fog(tiles: Array, player_capital: Vector2i, width: int, height: int) -> void:
	for r in height:
		for q in width:
			var distance := HexGeometry.wrap_distance(Vector2i(q, r), player_capital, width)
			var tile: Dictionary = tiles[r][q]
			tile.fog = "visible" if distance <= 4 else ("remembered" if distance <= 8 else "unknown")

static func cell_noise(q: int, r: int, seed: int) -> int:
	var value := posmod(seed + q * 1103515245 + r * 12345 + q * r * 97, 2147483647)
	value = posmod(value * 48271 + 1, 2147483647)
	return value % 101

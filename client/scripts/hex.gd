extends RefCounted
## Hex math: pointy-top hexagons, odd-r offset coordinates (cell.x = column, cell.y = row),
## horizontal cylinder wrap (columns wrap, rows do not). Mirrors docs/sdd/02-hex-e-mapa.md.

const SQRT3 := 1.7320508075688772
## Edge order is clockwise starting at north-east; it matches assets/tiles/manifest.json.
const EDGES: Array[String] = ["ne", "e", "se", "sw", "w", "nw"]

const _EVEN_ROW: Array[Vector2i] = [Vector2i(0, -1), Vector2i(1, 0), Vector2i(0, 1), Vector2i(-1, 1), Vector2i(-1, 0), Vector2i(-1, -1)]
const _ODD_ROW: Array[Vector2i] = [Vector2i(1, -1), Vector2i(1, 0), Vector2i(1, 1), Vector2i(0, 1), Vector2i(-1, 0), Vector2i(0, -1)]


## Neighbor offsets in EDGES order for a row (odd rows are shifted half a hex).
static func edge_offsets(row: int) -> Array[Vector2i]:
	return _ODD_ROW if (row & 1) == 1 else _EVEN_ROW


## Dense row-major tile index (the server's `TileIndex`) <-> odd-r offset cell.
static func cell_of_index(index: int, width: int) -> Vector2i:
	return Vector2i(index % width, index / width)


static func index_of_cell(cell: Vector2i, width: int) -> int:
	return cell.y * width + cell.x


static func wrap_col(col: int, width: int) -> int:
	return posmod(col, width)


static func in_rows(cell: Vector2i, height: int) -> bool:
	return cell.y >= 0 and cell.y < height


## Neighbors in EDGES order. Entries beyond the poles are skipped, so the result may be shorter than 6.
static func neighbors(cell: Vector2i, width: int, height: int) -> Array[Vector2i]:
	var offsets: Array[Vector2i] = _ODD_ROW if (cell.y & 1) == 1 else _EVEN_ROW
	var result: Array[Vector2i] = []
	for offset in offsets:
		var candidate: Vector2i = cell + offset
		if in_rows(candidate, height):
			result.append(Vector2i(wrap_col(candidate.x, width), candidate.y))
	return result


static func to_axial(cell: Vector2i) -> Vector2i:
	return Vector2i(cell.x - (cell.y - (cell.y & 1)) / 2, cell.y)


## Distance on an unwrapped plane.
static func distance(a: Vector2i, b: Vector2i) -> int:
	var aa := to_axial(a)
	var bb := to_axial(b)
	var dq := aa.x - bb.x
	var dr := aa.y - bb.y
	return (absi(dq) + absi(dr) + absi(dq + dr)) / 2


## Shortest distance taking the horizontal wrap into account.
static func wrap_distance(a: Vector2i, b: Vector2i, width: int) -> int:
	var best := 1 << 30
	for shift in [-width, 0, width]:
		best = mini(best, distance(a, Vector2i(b.x + shift, b.y)))
	return best


## Center of a cell in world pixels for hexes of circumradius `radius`.
static func center(cell: Vector2i, radius: float) -> Vector2:
	return Vector2(radius * SQRT3 * (cell.x + 0.5 * (cell.y & 1)), radius * 1.5 * cell.y)


## World pixel -> cell. Returns (-1, -1) when the point is outside the map rows.
static func pixel_to_cell(point: Vector2, radius: float, width: int, height: int) -> Vector2i:
	var row_guess := roundi(point.y / (radius * 1.5))
	var best := Vector2i(-1, -1)
	var best_d := INF
	for row in range(maxi(0, row_guess - 1), mini(height, row_guess + 2)):
		var col_guess := roundi(point.x / (radius * SQRT3) - 0.5 * (row & 1))
		for col in range(col_guess - 1, col_guess + 2):
			var d := point.distance_squared_to(center(Vector2i(col, row), radius))
			if d < best_d:
				best_d = d
				best = Vector2i(wrap_col(col, width), row)
	if best_d > radius * radius:
		return Vector2i(-1, -1)
	return best

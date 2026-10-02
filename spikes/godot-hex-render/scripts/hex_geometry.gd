class_name HexGeometry
extends RefCounted

const SQRT_3 := 1.7320508075688772

static func wrap_q(q: int, width: int) -> int:
	return posmod(q, width)

static func is_valid(cell: Vector2i, height: int) -> bool:
	return cell.y >= 0 and cell.y < height

static func neighbors(cell: Vector2i, width: int, height: int) -> Array[Vector2i]:
	# odd-r offset coordinates, clockwise: NE, E, SE, SW, W, NW.
	var offsets: Array[Vector2i] = []
	if cell.y % 2 == 0:
		offsets.assign([Vector2i(0, -1), Vector2i(1, 0), Vector2i(0, 1), Vector2i(-1, 1), Vector2i(-1, 0), Vector2i(-1, -1)])
	else:
		offsets.assign([Vector2i(1, -1), Vector2i(1, 0), Vector2i(1, 1), Vector2i(0, 1), Vector2i(-1, 0), Vector2i(0, -1)])
	var result: Array[Vector2i] = []
	for offset in offsets:
		var candidate: Vector2i = cell + offset
		if is_valid(candidate, height):
			candidate.x = wrap_q(candidate.x, width)
			if not result.has(candidate):
				result.append(candidate)
	return result

static func offset_to_axial(cell: Vector2i) -> Vector2i:
	return Vector2i(cell.x - (cell.y - (cell.y & 1)) / 2, cell.y)

static func hex_distance(a: Vector2i, b: Vector2i) -> int:
	var da := offset_to_axial(a)
	var db := offset_to_axial(b)
	var dq := da.x - db.x
	var dr := da.y - db.y
	return (abs(dq) + abs(dr) + abs(dq + dr)) / 2

static func wrap_distance(a: Vector2i, b: Vector2i, width: int) -> int:
	var best := 1 << 30
	for shift in [-width, 0, width]:
		best = min(best, hex_distance(a, Vector2i(b.x + shift, b.y)))
	return best

static func cell_center(cell: Vector2i, radius: float) -> Vector2:
	return Vector2(radius * SQRT_3 * (cell.x + 0.5 * (cell.y & 1)), radius * 1.5 * cell.y)

static func pixel_to_cell(point: Vector2, radius: float, width: int, height: int) -> Vector2i:
	# Round to the nearest odd-r center. This avoids fragile edge rounding and keeps wrap explicit.
	var approximate_row := roundi(point.y / (radius * 1.5))
	var best := Vector2i(0, clampi(approximate_row, 0, height - 1))
	var best_distance := INF
	for row in range(maxi(0, approximate_row - 2), mini(height, approximate_row + 3)):
		var approximate_col := roundi(point.x / (radius * SQRT_3) - 0.5 * (row & 1))
		for col in range(approximate_col - 2, approximate_col + 3):
			var candidate := Vector2i(wrap_q(col, width), row)
			var d := point.distance_squared_to(cell_center(Vector2i(col, row), radius))
			if d < best_distance:
				best_distance = d
				best = candidate
	return best

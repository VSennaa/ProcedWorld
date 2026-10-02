class_name MapGenerator
extends RefCounted

const BIOMES := ["oceano", "costa", "planicie", "floresta", "deserto", "montanha", "geleira"]

static func generate(width: int, height: int, seed: int) -> Array:
	var map: Array = []
	for r in height:
		var row: Array[String] = []
		for q in width:
			# Integer-only, fixed-order presentation port: no global RNG or floating state.
			var noise := cell_noise(q, r, seed)
			var continent := cell_noise(q / 5, r / 5, seed ^ 0x5f3759df)
			var latitude: int = int(abs(r * 2 - (height - 1)) * 100 / (height - 1))
			var elevation: int = noise + continent - 80
			var biome := "oceano"
			if latitude > 84:
				biome = "geleira"
			elif elevation < 18:
				biome = "oceano"
			elif elevation < 42:
				biome = "costa"
			elif elevation > 130:
				biome = "montanha"
			elif latitude > 65:
				biome = "planicie"
			elif noise > 70:
				biome = "floresta"
			elif noise < 28:
				biome = "deserto"
			else:
				biome = "planicie"
			row.append(biome)
		map.append(row)
	return map

static func cell_noise(q: int, r: int, seed: int) -> int:
	var value := posmod(seed + q * 1103515245 + r * 12345 + q * r * 97, 2147483647)
	value = posmod(value * 48271 + 1, 2147483647)
	return value % 101

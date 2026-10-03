extends RefCounted
## Remembers the server URL and the per-seat session tokens (reconnection) in user://.
## The token is a game-seat reconnect secret, not an account credential or an API key.

const DEFAULT_PATH := "user://sessions.cfg"


static func _key(world_id: int, civ: int) -> String:
	return "%d_%d" % [world_id, civ]


static func save_token(world_id: int, civ: int, token: String, path: String = DEFAULT_PATH) -> void:
	var cfg := ConfigFile.new()
	cfg.load(path)
	cfg.set_value("tokens", _key(world_id, civ), token)
	cfg.save(path)


static func load_token(world_id: int, civ: int, path: String = DEFAULT_PATH) -> String:
	var cfg := ConfigFile.new()
	if cfg.load(path) != OK:
		return ""
	return String(cfg.get_value("tokens", _key(world_id, civ), ""))


static func forget_token(world_id: int, civ: int, path: String = DEFAULT_PATH) -> void:
	var cfg := ConfigFile.new()
	if cfg.load(path) != OK:
		return
	if cfg.has_section_key("tokens", _key(world_id, civ)):
		cfg.erase_section_key("tokens", _key(world_id, civ))
		cfg.save(path)


static func save_last(url: String, world_id: int, civ: int, path: String = DEFAULT_PATH) -> void:
	var cfg := ConfigFile.new()
	cfg.load(path)
	cfg.set_value("last", "url", url)
	cfg.set_value("last", "world_id", world_id)
	cfg.set_value("last", "civ", civ)
	cfg.save(path)


## {url, world_id, civ} of the last session, with defaults when nothing was saved.
static func load_last(default_url: String, path: String = DEFAULT_PATH) -> Dictionary:
	var cfg := ConfigFile.new()
	cfg.load(path)
	return {
		"url": String(cfg.get_value("last", "url", default_url)),
		"world_id": int(cfg.get_value("last", "world_id", 1)),
		"civ": int(cfg.get_value("last", "civ", 0)),
	}

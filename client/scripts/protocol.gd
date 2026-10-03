extends RefCounted
## Protocol client helpers (docs/sdd/10-protocolo.md): JSON envelope parsing and building.
## No networking here, so it is fully testable headless.

const PROTOCOL_VERSION := "1.0"
const SUPPORTED_MAJOR := 1


## Parses an envelope from JSON text. Returns {ok, error, envelope}.
## The envelope has protocol_version, request_id, type and payload. Unknown fields are ignored.
static func parse_envelope(text: String) -> Dictionary:
	var json := JSON.new()
	if json.parse(text) != OK:
		return _fail("JSON invalido: %s" % json.get_error_message())
	return validate_envelope(json.data)


static func validate_envelope(data: Variant) -> Dictionary:
	if typeof(data) != TYPE_DICTIONARY:
		return _fail("envelope nao e um objeto")
	var version: Variant = data.get("protocol_version")
	if typeof(version) != TYPE_STRING:
		return _fail("protocol_version ausente")
	if not is_version_compatible(version):
		return _fail("versao incompativel: %s (suportada: %s)" % [version, PROTOCOL_VERSION])
	for field in ["request_id", "type"]:
		if typeof(data.get(field)) != TYPE_STRING or String(data[field]).is_empty():
			return _fail("campo obrigatorio ausente: %s" % field)
	if typeof(data.get("payload")) != TYPE_DICTIONARY:
		return _fail("payload ausente ou nao e um objeto")
	return {
		"ok": true,
		"error": "",
		"envelope": {
			"protocol_version": version,
			"request_id": data["request_id"],
			"type": data["type"],
			"payload": data["payload"],
		},
	}


## Same major version is compatible; a newer minor only adds optional fields.
static func is_version_compatible(version: String) -> bool:
	var parts := version.split(".")
	return parts.size() >= 2 and parts[0].is_valid_int() and parts[1].is_valid_int() and parts[0].to_int() == SUPPORTED_MAJOR


static func build_envelope(type: String, payload: Dictionary, request_id: String) -> String:
	return JSON.stringify({
		"protocol_version": PROTOCOL_VERSION,
		"request_id": request_id,
		"type": type,
		"payload": payload,
	})


static func _fail(message: String) -> Dictionary:
	return {"ok": false, "error": message, "envelope": {}}

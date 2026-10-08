extends RefCounted
## Session-local chronicle (brief C2c). The server does not persist a chronicle yet, so this holds
## only the turns received while the client stays connected; every new session rebuilds it from
## scratch. This class accumulates raw facts and keeps them engine-shaped: the PT-BR text is
## produced by chronicle_view.gd. Nothing here computes game rules.

## Turn at which the current session started, or -1 before the first view.
var session_start_turn := -1
## Turns received, oldest first. Each entry:
## {turn: int, events: Array, commands: Dictionary, responses: Array, cities: Dictionary}.
var entries: Array = []
## Entropy events seen in earlier views, so a later response stays readable after the event leaves
## the projection: event id -> {title: String, choices: {choice_id: label}}.
var event_titles: Dictionary = {}


## Marks the first turn of the session; a later resync snapshot never moves it.
func record_start(turn: int) -> void:
	if session_start_turn < 0:
		session_start_turn = turn


## Remembers the readable title and option labels of pending Entropy events, so the response stays
## readable once the event is resolved and leaves the projection.
func remember_events(agenda: Array) -> void:
	for card in agenda:
		if typeof(card) != TYPE_DICTIONARY or not card.has("event_id"):
			continue
		var choices: Dictionary = {}
		for option in card.get("options", []):
			if typeof(option) == TYPE_DICTIONARY:
				choices[String(option.get("id", ""))] = String(option.get("label", ""))
		event_titles[int(card["event_id"])] = {"title": String(card.get("title", "")), "choices": choices}


## Appends one resolved turn from a `turn_diff` payload. `cities` maps city id -> name as of the
## moment the turn was received (a foreign city may be forgotten later). A repeated diff for a
## turn already recorded is ignored, so a replayed resync never duplicates history.
func record_turn(payload: Dictionary, cities: Dictionary) -> void:
	var turn := int(payload.get("from_turn", payload.get("turn", 0)))
	for entry in entries:
		if int(entry["turn"]) == turn:
			return
	var commands: Dictionary = {}
	var raw_commands: Variant = payload.get("commands", [])
	if typeof(raw_commands) == TYPE_ARRAY:
		for command in raw_commands:
			if typeof(command) == TYPE_DICTIONARY:
				commands[int(command.get("command_id", -1))] = command
	var responses: Array = []
	for command in commands.values():
		if String(command.get("kind", "")) == "respond_to_event":
			responses.append(_response(command))
	entries.append({
		"turn": turn,
		"events": payload.get("events", []).duplicate(true) if typeof(payload.get("events")) == TYPE_ARRAY else [],
		"commands": commands,
		"responses": responses,
		"cities": cities.duplicate(),
	})


func _response(command: Dictionary) -> Dictionary:
	var payload: Dictionary = command.get("payload", {}) if typeof(command.get("payload")) == TYPE_DICTIONARY else {}
	var data: Dictionary = payload.get("data", {}) if typeof(payload.get("data")) == TYPE_DICTIONARY else {}
	var event_id := int(data.get("event_id", -1))
	var choice_id := String(data.get("choice_id", ""))
	var known: Dictionary = event_titles.get(event_id, {})
	var choices: Dictionary = known.get("choices", {})
	return {
		"event_id": event_id,
		"title": String(known.get("title", "")),
		"choice": choice_id,
		"label": String(choices.get(choice_id, "")),
	}

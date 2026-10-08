extends MarginContainer
## Crônica screen (brief C2c): the turns received in this session, read-only. The server does not
## persist a chronicle yet, so the screen is explicitly labelled "this session" and the model is
## rebuilt on reconnection. Facts come straight from the server report (`events_for`); the client
## only translates them to PT-BR and never recomputes effects.

const ServerView := preload("res://scripts/server_view.gd")

const MUTED := Color("b9c1c8")
const CARD_BG := Color("223140")

## PT-BR descriptions of the engine `CommandKind` (own commands only).
const COMMAND_LABELS := {
	"end_turn": "encerrar o turno", "move_unit": "mover unidade", "found_city": "fundar cidade",
	"set_city_focus": "definir foco da cidade", "set_research": "escolher pesquisa",
	"set_research_investment": "definir investimento em pesquisa", "activate_practice": "ativar prática",
	"deactivate_practice": "desativar prática", "queue_unit": "enfileirar unidade",
	"remove_queued_unit": "remover da fila", "move_queued_unit": "reordenar a fila",
	"declare_attack": "atacar", "explore": "explorar", "set_unit_order": "dar ordem à unidade",
	"skip_unit": "pular unidade", "keep_plan": "manter o plano", "propose_diplomacy": "propor diplomacia",
	"break_treaty": "romper tratado", "declare_war": "declarar guerra",
	"apply_event": "evento da Entropia", "respond_to_event": "responder a evento",
}
## PT-BR of the engine `RejectionReason` shown by a rejected command in the report.
const REJECTION_LABELS := {
	"wrong_world": "mundo errado", "wrong_turn": "turno errado",
	"duplicate_accepted_sequence": "ordem repetida", "kind_payload_mismatch": "ordem e dados não combinam",
	"unknown_civilization": "civilização desconhecida", "unknown_unit": "unidade desconhecida",
	"unknown_city": "cidade desconhecida", "not_command_owner": "não pertence à sua civilização",
	"invalid_tile": "tile inválido", "destination_occupied": "destino ocupado",
	"city_already_exists": "cidade já existe", "city_tile_occupied": "já há uma cidade nesse tile",
	"ruleset_mismatch": "regras incompatíveis", "civilization_frozen": "civilização congelada",
	"unknown_technology": "tecnologia desconhecida", "missing_technology_prerequisite": "faltam pré-requisitos",
	"invalid_research_investment": "investimento inválido", "unknown_practice": "prática desconhecida",
	"practice_technology_not_researched": "tecnologia da prática não dominada",
	"practice_limit_reached": "limite de práticas atingido", "unknown_unit_type": "tipo de unidade desconhecido",
	"unit_technology_not_researched": "tecnologia da unidade não dominada",
	"unit_already_reserved": "unidade já reservada", "invalid_queue_index": "item da fila inválido",
	"missing_settler": "falta um colono", "attack_not_hostile": "alvo não é hostil",
	"attack_out_of_range": "alvo fora de alcance", "unit_id_exhausted": "limite de unidades atingido",
	"turn_overflow": "limite de turnos atingido", "mandate_required": "mandato necessário",
	"mandate_violation": "violação do mandato", "invalid_diplomatic_transition": "transição diplomática inválida",
	"unknown_war_objective": "objetivo de guerra desconhecido", "missing_casus_belli": "falta justificativa",
	"repeated_proposal_blocked": "proposta repetida bloqueada", "ungrounded_diplomacy": "diplomacia sem fundamento",
	"self_diplomacy": "diplomacia consigo mesmo", "event_forbidden": "evento proibido",
	"event_invalid": "evento inválido", "event_on_cooldown": "evento em espera",
	"event_over_budget": "evento acima do orçamento", "event_protected": "alvo protegido",
	"event_busy": "alvo ocupado", "event_unsafe_target": "alvo inseguro",
	"event_no_useful_response": "sem resposta útil", "event_unknown": "evento desconhecido",
	"event_choice_invalid": "resposta inválida", "event_unaffordable": "resposta impagável",
	"not_a_worker": "não é trabalhador", "unknown_improvement": "melhoria desconhecida",
	"improvement_technology_not_researched": "tecnologia da melhoria não dominada",
	"improvement_terrain_invalid": "melhoria inválida neste terreno", "tile_already_improved": "tile já melhorado",
	"tile_work_in_progress": "obra já em andamento", "tile_outside_territory": "tile fora do território",
	"city_tile_not_improvable": "não dá para melhorar o tile da cidade",
}
const PROPOSAL_LABELS := {
	"trade": "comércio", "passage": "passagem", "aid": "ajuda", "pact": "pacto",
	"alliance": "aliança", "reparation": "reparação", "truce": "trégua",
}
const OUTCOME_LABELS := {
	"accepted": "aceita", "refused": "recusada", "undecided": "indecisa", "executed": "executada",
}


static func command_label(command: Dictionary) -> String:
	var kind := String(command.get("kind", ""))
	if kind.is_empty():
		return "ordem"
	return COMMAND_LABELS.get(kind, ServerView.humanize(kind))


static func rejection_label(reason: String) -> String:
	if reason.is_empty():
		return "motivo desconhecido"
	return REJECTION_LABELS.get(reason, ServerView.humanize(reason))


static func city_name(cities: Dictionary, id: int) -> String:
	if id < 0:
		return "cidade desconhecida"
	return String(cities.get(id, "Cidade %d" % (id + 1)))


## One PT-BR line for a report event (`events_for`). Returns "" when the fact is carried elsewhere
## (an Entropy response is shown by `response_line`, a diplomatic result by `diplomacy_line`).
static func event_line(event: Dictionary, entry: Dictionary) -> String:
	var kind := String(event.get("type", ""))
	var data: Dictionary = event.get("data", {}) if typeof(event.get("data")) == TYPE_DICTIONARY else {}
	match kind:
		"command_applied":
			var command_id := int(data.get("command_id", -1))
			var applied: Dictionary = entry["commands"].get(command_id, {})
			if String(applied.get("kind", "")) == "respond_to_event" or _has_diplomacy(entry["events"], command_id):
				return ""
			return "Ordem aplicada: %s." % command_label(applied)
		"command_rejected":
			var rejected: Dictionary = entry["commands"].get(int(data.get("command_id", -1)), {})
			return "Ordem recusada: %s — %s." % [command_label(rejected), rejection_label(String(data.get("reason", "")))]
		"population_migrated":
			return "População migrou de %s para %s." % [city_name(entry["cities"], int(data.get("from", -1))), city_name(entry["cities"], int(data.get("to", -1)))]
		"collapse_triggered":
			return "Colapso da Civilização %d." % (int(data.get("civilization", -1)) + 1)
		"diplomacy_resolved":
			return diplomacy_line(data, int(entry.get("viewer", -1)))
		_:
			return "Fato: %s." % ServerView.humanize(kind)


## True when the same command also produced a `diplomacy_resolved` event: the applied order and the
## resolution are one fact, and `diplomacy_line` already carries it.
static func _has_diplomacy(events: Array, command_id: int) -> bool:
	for event in events:
		if typeof(event) != TYPE_DICTIONARY or String(event.get("type", "")) != "diplomacy_resolved":
			continue
		var data: Dictionary = event.get("data", {}) if typeof(event.get("data")) == TYPE_DICTIONARY else {}
		if int(data.get("command_id", -1)) == command_id:
			return true
	return false


## "você propôs um acordo de comércio a Civilização 2 — aceita (Contato → Paz)". The event's `actor`
## names who acted, so the recipient is never read as if it had proposed.
static func diplomacy_line(data: Dictionary, viewer: int) -> String:
	var action: Dictionary = data.get("action", {}) if typeof(data.get("action")) == TYPE_DICTIONARY else {}
	var resolution: Dictionary = data.get("resolution", {}) if typeof(data.get("resolution")) == TYPE_DICTIONARY else {}
	var actor := int(data.get("actor", -1))
	var other := int(data.get("other", -1))
	var who := "você" if (actor == viewer and viewer >= 0) else "Civilização %d" % (actor + 1)
	var target := "você" if (other == viewer and viewer >= 0) else "Civilização %d" % (other + 1)
	var line := "%s %s" % [who, _action_text(action, target)]
	var outcome: String = OUTCOME_LABELS.get(String(resolution.get("outcome", "")), "")
	if not outcome.is_empty():
		line += " — %s" % outcome
	var from_state := ServerView.relation_state_label(String(resolution.get("from", "")))
	var to_state := ServerView.relation_state_label(String(resolution.get("to", "")))
	if not to_state.is_empty() and from_state != to_state:
		line += " (%s → %s)" % [from_state, to_state]
	return line + "."


static func _action_text(action: Dictionary, target: String) -> String:
	var data: Dictionary = action.get("data", {}) if typeof(action.get("data")) == TYPE_DICTIONARY else {}
	match String(action.get("type", "")):
		"propose":
			return "propôs um acordo de %s a %s" % [PROPOSAL_LABELS.get(String(data.get("kind", "")), "cooperação"), target]
		"break_treaty":
			return "rompeu um tratado com %s" % target
		"declare_war":
			return "declarou guerra a %s" % target
		_:
			return ServerView.humanize(String(action.get("type", "")))


static func response_line(response: Dictionary) -> String:
	var title := String(response.get("title", ""))
	if title.is_empty():
		title = "evento %d" % int(response.get("event_id", 0))
	var label := String(response.get("label", ""))
	if label.is_empty():
		label = String(response.get("choice", "?"))
	return "Entropia: respondeu a “%s” com “%s”." % [title, label]


func build(chronicle: RefCounted) -> void:
	for child in get_children():
		child.queue_free()
	for side in ["left", "right", "top", "bottom"]:
		add_theme_constant_override("margin_" + side, 12)
	var scroll := ScrollContainer.new()
	scroll.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_DISABLED
	add_child(scroll)
	var root := VBoxContainer.new()
	root.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	root.add_theme_constant_override("separation", 12)
	scroll.add_child(root)
	_add_title(root, "Crônica")
	_add_note(root, "Somente esta sessão: o servidor ainda não guarda a Crônica. Sair ou reconectar reinicia este histórico.")
	if chronicle.session_start_turn >= 0:
		_add_note(root, "Sessão iniciada no turno %d." % chronicle.session_start_turn)
	var entries: Array = chronicle.entries
	if entries.is_empty():
		_add_note(root, "Nenhum turno resolvido nesta sessão. Os fatos aparecem aqui quando um turno for resolvido.")
		return
	for i in range(entries.size() - 1, -1, -1):
		root.add_child(_build_turn(entries[i]))


func _build_turn(entry: Dictionary) -> Control:
	var panel := PanelContainer.new()
	var style := StyleBoxFlat.new()
	style.bg_color = CARD_BG
	style.set_corner_radius_all(10)
	style.set_content_margin_all(16)
	panel.add_theme_stylebox_override("panel", style)
	var box := VBoxContainer.new()
	box.add_theme_constant_override("separation", 6)
	panel.add_child(box)
	var title := Label.new()
	title.text = "Turno %d" % int(entry["turn"])
	title.add_theme_font_size_override("font_size", 24)
	box.add_child(title)
	var lines: Array = []
	for event in entry["events"]:
		if typeof(event) == TYPE_DICTIONARY:
			var line := event_line(event, entry)
			if not line.is_empty():
				lines.append(line)
	for response in entry["responses"]:
		if typeof(response) == TYPE_DICTIONARY:
			lines.append(response_line(response))
	if lines.is_empty():
		lines.append("Sem fatos registrados neste turno.")
	for line in lines:
		var label := Label.new()
		label.text = line
		label.add_theme_font_size_override("font_size", 19)
		label.add_theme_color_override("font_color", MUTED)
		label.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
		box.add_child(label)
	return panel


func _add_title(parent: Control, text: String) -> void:
	var label := Label.new()
	label.text = text
	label.add_theme_font_size_override("font_size", 28)
	parent.add_child(label)


func _add_note(parent: Control, text: String) -> void:
	var label := Label.new()
	label.text = text
	label.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	label.add_theme_font_size_override("font_size", 17)
	label.add_theme_color_override("font_color", MUTED)
	parent.add_child(label)

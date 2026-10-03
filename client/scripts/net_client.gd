extends Node
## Thin WebSocket wrapper for the authoritative server. Optional at runtime: the app works
## from the fixture when this is never connected, and the tests never touch it.

const Protocol := preload("res://scripts/protocol.gd")
const DEFAULT_URL := "ws://127.0.0.1:8100/ws"

signal envelope_received(envelope: Dictionary)
signal protocol_error(message: String)
signal connection_changed(is_open: bool)

var _peer := WebSocketPeer.new()
var _was_open := false
var _active := false
var _counter := 0


func connect_to_server(url: String = DEFAULT_URL) -> Error:
	_active = true
	return _peer.connect_to_url(url)


func is_open() -> bool:
	return _peer.get_ready_state() == WebSocketPeer.STATE_OPEN


func send_message(type: String, payload: Dictionary) -> bool:
	if not is_open():
		return false
	_counter += 1
	_peer.send_text(Protocol.build_envelope(type, payload, "c-%d" % _counter))
	return true


func _process(_delta: float) -> void:
	if not _active:
		return
	_peer.poll()
	var open := is_open()
	if open != _was_open:
		_was_open = open
		connection_changed.emit(open)
	while open and _peer.get_available_packet_count() > 0:
		var parsed := Protocol.parse_envelope(_peer.get_packet().get_string_from_utf8())
		if parsed["ok"]:
			envelope_received.emit(parsed["envelope"])
		else:
			protocol_error.emit(parsed["error"])

extends Node
## Thin WebSocket wrapper for the authoritative server. Optional at runtime: the app works
## from the fixture when this is never connected, and the tests never touch it.

const Protocol := preload("res://scripts/protocol.gd")
const DEFAULT_URL := "ws://127.0.0.1:8100/ws"
## A socket still connecting after this long is given up (e.g. nothing listening behind a tunnel).
const CONNECT_TIMEOUT_MS := 10000

signal envelope_received(envelope: Dictionary)
signal protocol_error(message: String)
signal connection_changed(is_open: bool)
## Emitted once when the socket ends; `was_open` is false when it never connected.
signal connection_lost(was_open: bool)

var _peer := WebSocketPeer.new()
var _was_open := false
var _ever_open := false
var _active := false
var _counter := 0
var _connect_deadline := 0


func connect_to_server(url: String = DEFAULT_URL) -> Error:
	_peer = WebSocketPeer.new()
	_was_open = false
	_ever_open = false
	_active = true
	_connect_deadline = Time.get_ticks_msec() + CONNECT_TIMEOUT_MS
	return _peer.connect_to_url(url)


func close() -> void:
	_active = false
	_peer.close()
	if _was_open:
		_was_open = false
		connection_changed.emit(false)


func is_open() -> bool:
	return _peer.get_ready_state() == WebSocketPeer.STATE_OPEN


## Sends a request and returns its request id, or "" when the socket is not open.
func send_request(type: String, payload: Dictionary) -> String:
	if not is_open():
		return ""
	_counter += 1
	var request_id := "c-%d" % _counter
	_peer.send_text(Protocol.build_envelope(type, payload, request_id))
	return request_id


func send_message(type: String, payload: Dictionary) -> bool:
	return send_request(type, payload) != ""


func _process(_delta: float) -> void:
	if not _active:
		return
	_peer.poll()
	var open := is_open()
	if open != _was_open:
		_was_open = open
		connection_changed.emit(open)
	if open:
		_ever_open = true
	if not _ever_open and _peer.get_ready_state() == WebSocketPeer.STATE_CONNECTING and Time.get_ticks_msec() > _connect_deadline:
		_peer.close()
		_active = false
		connection_lost.emit(false)
		return
	if _peer.get_ready_state() == WebSocketPeer.STATE_CLOSED:
		_active = false
		connection_lost.emit(_ever_open)
		return
	while open and _peer.get_available_packet_count() > 0:
		var parsed := Protocol.parse_envelope(_peer.get_packet().get_string_from_utf8())
		if parsed["ok"]:
			envelope_received.emit(parsed["envelope"])
		else:
			protocol_error.emit(parsed["error"])

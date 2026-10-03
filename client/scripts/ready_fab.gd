extends Button
## Compact floating "next decision / Pronto" button: an icon and a numeric badge with the remaining
## decisions. With decisions left it asks for the next one; with none it becomes the Pronto button;
## after Pronto it waits (disabled) until the turn advances. main.gd owns what each tap does.

signal next_requested
signal ready_requested

const SIZE := 76.0
const ICON_NEXT := preload("res://assets/icons/proxima-decisao.svg")
const ICON_READY := preload("res://assets/icons/pronto.svg")
const ICON_WAIT := preload("res://assets/icons/turno.svg")
const BADGE_COLOR := Color("d55e00")

var count := 0
var waiting := false
var _icon: TextureRect
var _badge: PanelContainer
var _badge_label: Label


func _init() -> void:
	custom_minimum_size = Vector2(SIZE, SIZE)
	size = Vector2(SIZE, SIZE)
	focus_mode = Control.FOCUS_NONE
	var style := StyleBoxFlat.new()
	style.bg_color = Color(0.063, 0.094, 0.125, 0.95)
	style.border_color = Color("e69f00")
	style.set_border_width_all(3)
	style.set_corner_radius_all(int(SIZE / 2.0))
	for state in ["normal", "hover", "pressed", "disabled", "focus"]:
		add_theme_stylebox_override(state, style)
	_icon = TextureRect.new()
	_icon.set_anchors_preset(Control.PRESET_FULL_RECT)
	_icon.offset_left = 12
	_icon.offset_top = 12
	_icon.offset_right = -12
	_icon.offset_bottom = -12
	_icon.expand_mode = TextureRect.EXPAND_IGNORE_SIZE
	_icon.stretch_mode = TextureRect.STRETCH_KEEP_ASPECT_CENTERED
	_icon.mouse_filter = Control.MOUSE_FILTER_IGNORE
	add_child(_icon)
	_badge = PanelContainer.new()
	var badge_style := StyleBoxFlat.new()
	badge_style.bg_color = BADGE_COLOR
	badge_style.set_corner_radius_all(14)
	badge_style.content_margin_left = 8
	badge_style.content_margin_right = 8
	badge_style.content_margin_top = 0
	badge_style.content_margin_bottom = 0
	_badge.add_theme_stylebox_override("panel", badge_style)
	_badge.mouse_filter = Control.MOUSE_FILTER_IGNORE
	_badge.custom_minimum_size = Vector2(28, 28)
	_badge.position = Vector2(SIZE - 34, -6)
	_badge_label = Label.new()
	_badge_label.add_theme_font_size_override("font_size", 20)
	_badge_label.add_theme_color_override("font_color", Color.WHITE)
	_badge_label.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
	_badge_label.mouse_filter = Control.MOUSE_FILTER_IGNORE
	_badge.add_child(_badge_label)
	add_child(_badge)
	pressed.connect(_on_pressed)
	set_state(0, false)


## `decisions`: remaining decisions; `is_waiting`: Pronto was sent and the turn has not advanced.
func set_state(decisions: int, is_waiting: bool) -> void:
	count = decisions
	waiting = is_waiting
	disabled = waiting
	_badge.visible = decisions > 0 and not waiting
	_badge_label.text = str(decisions)
	if waiting:
		_icon.texture = ICON_WAIT
		tooltip_text = "Pronto enviado: aguardando o turno avançar"
	elif decisions > 0:
		_icon.texture = ICON_NEXT
		tooltip_text = "Próxima decisão (%d restantes)" % decisions
	else:
		_icon.texture = ICON_READY
		tooltip_text = "Pronto: encerrar o turno"
	modulate = Color(1, 1, 1, 0.6) if waiting else Color.WHITE


func _on_pressed() -> void:
	if waiting:
		return
	if count > 0:
		next_requested.emit()
	else:
		ready_requested.emit()

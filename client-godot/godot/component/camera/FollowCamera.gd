extends Camera2D

var _anchor := Vector2.ZERO
var wish_anchor := Vector2.ZERO

var enable_look := true
var _look_dir := Vector2.ZERO

var wish_zoom := 1.0

func _unhandled_input(event: InputEvent) -> void:
	if event.is_action_pressed("game_zoom_in"):
		zoom_in()
	elif event.is_action_pressed("game_zoom_out"):
		zoom_out()
	elif event.is_action_pressed("ui_home"):
		wish_zoom = 1.0

func _process(_delta: float) -> void:
	_anchor = _anchor * 0.9 + wish_anchor * 0.1
	
	if enable_look:
		var vp := get_tree().get_root()
		var mouse_look := vp.get_mouse_position() - vp.size * 0.5
		_look_dir = _look_dir * 0.98 + mouse_look * 0.02
	else:
		_look_dir = Vector2.ZERO
	
	position = _anchor + _look_dir * zoom.x
	
	zoom.x = zoom.x * 0.9 + wish_zoom * 0.1
	if abs(zoom.x - wish_zoom) < 0.01:
		zoom.x = wish_zoom
	zoom.y = zoom.x

func zoom_in() -> void:
	wish_zoom = max(stepify(wish_zoom - 0.5, 0.5), 0.5)

func zoom_out() -> void:
	wish_zoom = min(stepify(wish_zoom + 0.5, 0.5), 8.0)

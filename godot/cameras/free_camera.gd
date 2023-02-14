extends Camera2D

const ZOOM_MAX := 4.0
const ZOOM_MIN := 0.125

var wish_zoom := 1.0
var _tween :Tween
var _zoom_before := Vector2.ONE

func _unhandled_input(event: InputEvent) -> void:
	if event.is_action_pressed("camera_zoom_in"):
		get_viewport().set_input_as_handled()
		_zoom(1.1)
	elif event.is_action_pressed("camera_zoom_out"):
		get_viewport().set_input_as_handled()
		_zoom(0.91)
	elif event.is_action_pressed("ui_home"):
		get_viewport().set_input_as_handled()
		_zoom(1.0 / zoom.x)
	elif event.is_action_pressed("camera_drag"):
		get_viewport().set_input_as_handled()
		_kill_tween()
	elif event is InputEventMouseMotion && Input.is_action_pressed("camera_drag"):
		position -= event.get_relative() / zoom

func _zoom(zoom_change: float) -> void:
	if Input.is_action_pressed("camera_drag"):
		return
	
	wish_zoom = zoom.x * zoom_change
	wish_zoom = clampf(wish_zoom, ZOOM_MIN, ZOOM_MAX)
	
	var current_offset = get_global_mouse_position() - position
	var final_offset = current_offset * (zoom / wish_zoom)
	var diff = final_offset - current_offset
	var final_position = position - diff
	
	zoom = Vector2(wish_zoom, wish_zoom)
	position = final_position
	
#	_zoom_before = get_zoom()
#
#	_make_tween()
#	_tween.tween_property(self, ^"zoom", Vector2(wish_zoom, wish_zoom), 0.5)
#	_tween.tween_method(_zoom_toward_cursor.bind(get_local_mouse_position(), zoom, get_screen_center_position()), zoom.x, wish_zoom, 0.5)
#	_tween.tween_method(_zoom_toward_cursor2.bind(get_global_mouse_position()), zoom.x, wish_zoom, 0.5)
#	_tween.tween_property(self, ^"position", final_position, 0.5)

func _zoom_toward_cursor2(new_zoom: float, zoom_toward: Vector2) -> void:
	zoom = Vector2(new_zoom, new_zoom)
	
	var new_position := (position - zoom_toward) * _zoom_before / zoom + zoom_toward
	position = new_position
	_zoom_before = zoom

func _zoom_toward_cursor(new_zoom: float, target: Vector2, start_zoom: Vector2, start_position: Vector2) -> void:
	zoom = Vector2(new_zoom, new_zoom)
	
	var change := target * (start_zoom / zoom)
	var diff := change - target
	position = start_position - diff

func _kill_tween() -> void:
	if _tween:
		_tween.kill()

func _make_tween() -> void:
	_kill_tween()
	_tween = create_tween().set_trans(Tween.TRANS_CUBIC).set_ease(Tween.EASE_OUT).set_parallel(true)

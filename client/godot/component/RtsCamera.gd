extends Camera2D

var last_mouse_pos := Vector2.ZERO

var zoom_before := Vector2.ONE
var wish_zoom := 1.0
var zoom_toward := Vector2.ZERO
onready var tween := Tween.new()

func _ready() -> void:
	add_child(tween)

func _unhandled_input(event: InputEvent) -> void:
	if event.is_action_pressed("game_zoom_in"):
		zoom_tween(0.80)
	elif event.is_action_pressed("game_zoom_out"):
		zoom_tween(1.25)
	elif event.is_action_pressed("ui_home"):
		position = Vector2.ZERO
		zoom = Vector2.ONE

func _process(_delta: float) -> void:
	if Input.is_action_just_pressed("game_camera_drag"):
		last_mouse_pos = get_local_mouse_position()
	elif Input.is_action_pressed("game_camera_drag"):
		# Drag camera.
		var next_pos = get_local_mouse_position()
		var dif = next_pos - last_mouse_pos
		position -= dif
		last_mouse_pos = next_pos
	elif tween.is_active():
		# Keep the distance between cursor when zoom was started 
		# and camera.position the same by moving camera.
		# Aka zoom toward/away from cursor.
		position = (position - zoom_toward) * zoom / zoom_before + zoom_toward
		zoom_before = get_zoom()

func zoom_tween(zoom_change: float) -> void:
	wish_zoom *= zoom_change
	wish_zoom = stepify(wish_zoom, 0.15)
	wish_zoom = clamp(wish_zoom, 0.15, 8.0)
	if abs(wish_zoom - 1.0) < 0.15:
		wish_zoom = 1.0
	elif abs(wish_zoom - 2.0) < 0.15:
		wish_zoom = 2.0
	elif abs(wish_zoom - 4.0) < 0.15:
		wish_zoom = 4.0
	elif abs(wish_zoom - 0.5) < 0.06:
		wish_zoom = 0.5
	
	zoom_toward = get_global_mouse_position()
	zoom_before = get_zoom()
	
	if !tween.remove_all():
		push_error("useless error")
	if wish_zoom == zoom.x:
		return
	if !tween.interpolate_property(self, "zoom", null, Vector2(wish_zoom, wish_zoom), 0.6, Tween.TRANS_CUBIC, Tween.EASE_OUT):
		print_debug("hur dur")
	if !tween.start():
		push_error("more useless error")

extends Camera2D

var origin := Vector2.ZERO
var wish_origin := Vector2.ZERO

var wish_zoom := 1.0
onready var tween := Tween.new()

func _ready() -> void:
	add_child(tween)

func _unhandled_input(event: InputEvent) -> void:
	if event.is_action_pressed("game_zoom_in"):
		zoom_tween(0.80)
	elif event.is_action_pressed("game_zoom_out"):
		zoom_tween(1.25)
	elif event.is_action_pressed("ui_home"):
		zoom = Vector2.ONE

func _process(_delta: float) -> void:
	var mouse_pos = get_local_mouse_position()
	
	origin = origin * 0.96 + wish_origin * 0.04
	
	var look_dif = (mouse_pos - origin).clamped(4096.0)
	
	position = wish_origin

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
	
	if !tween.remove_all():
		push_error("useless error")
	if wish_zoom == zoom.x:
		return
	if !tween.interpolate_property(self, "zoom", null, Vector2(wish_zoom, wish_zoom), 0.6, Tween.TRANS_CUBIC, Tween.EASE_OUT):
		print_debug("hur dur")
	if !tween.start():
		push_error("more useless error")

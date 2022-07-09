extends Camera2D

var origin := Vector2.ZERO
var wish_origin := Vector2.ZERO
var look_dir := Vector2.ZERO

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
	var vp := get_tree().get_root()
	var uv = (vp.get_mouse_position() / vp.size - Vector2(0.5, 0.5)) * Vector2(2.0, 2.0)
	look_dir = look_dir * 0.98 + uv * 0.02
	var look_dif = look_dir * Vector2(4096.0, 4096.0)
	
	origin = origin * 0.9 + wish_origin * 0.1
	
	position = origin + look_dif

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

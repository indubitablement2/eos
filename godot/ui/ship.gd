extends BaseButton

const COLOR_UNFOCUS := Color(0.98, 0.98, 1.0, 0.0)
const COLOR_FOCUS := Color(0.98, 0.98, 1.0, 0.7)
const COLOR_FOCUS_PULSE := Color(0.98, 0.98, 1.0, 0.45)

#@onready var _selected :CanvasItem = $Select
@onready var _destroyed :CanvasItem = $Destroyed
#@onready var _activated :CanvasItem = $Activated
@onready var _icon :TextureRect = $Icon
@onready var _cost_label :Label = $Cost

#var disabled := false : set = set_disabled

var tween :Tween = null


func set_ship(icon: Texture2D, size_factor: float, tooptip: String, cost: int, destroyed := false) -> void:
	_icon.set_texture(icon)
	_icon.anchor_left = 0.5 - size_factor * 0.5
	_icon.anchor_right = 0.5 + size_factor * 0.5
	set_tooltip_text(tooptip)
	set_cost(cost)
	if destroyed:
		set_destroyed()

func set_destroyed() -> void:
	disabled = true
	set_pressed(false)
	set_self_modulate(Color(0.5, 0.5, 0.5))
	_destroyed.set_visible(true)

func set_cost(cost: int) -> void:
	_cost_label.set_text(str(cost))

#func _on_focus_entered() -> void:
##	if Input.is_mouse_button_pressed(MOUSE_BUTTON_LEFT):
##		return
#
##	_selected.set_visible(true)
#	_make_tween()
#	tween.tween_property(_selected, ^"modulate", COLOR_FOCUS, 0.4)
#
#	await tween.finished
#	_make_tween()
#	tween.set_loops()
#	tween.tween_property(_selected, ^"modulate", COLOR_FOCUS_PULSE, 1.0)
#	tween.tween_property(_selected, ^"modulate", COLOR_FOCUS, 1.0)
#
#func _on_focus_exited() -> void:
#	_make_tween()
#	tween.tween_property(_selected, ^"modulate", COLOR_UNFOCUS, 0.4)
#	await tween.finished
#	_selected.set_visible(false)

func _make_tween() -> void:
	if tween:
		tween.kill()
	tween = create_tween().set_trans(Tween.TRANS_CUBIC).set_ease(Tween.EASE_OUT)


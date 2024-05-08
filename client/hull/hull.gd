extends Sprite2D
class_name Hull

static var HULL_SCENES : Array[PackedScene] = [
	load("res://hull/janitor/janitor.tscn"),
]

var hull_id : int

var _prev_position : Vector2
var _next_position := Vector2(NAN, NAN)
var _prev_rotation : float
var _next_rotation : float


func _process(_delta: float) -> void:
	position = _prev_position.lerp(_next_position, Simulation.node.sim_dt)
	rotation = lerp_angle(_prev_rotation, _next_rotation, Simulation.node.sim_dt)


func apply_state(
	position_delta: Vector2,
	rotation_delta: float) -> void:
	if _next_position.is_finite():
		_prev_position = _next_position
		_prev_rotation = _next_rotation
		_next_position = _prev_position + position_delta
		_next_rotation = _prev_rotation + rotation_delta
	else:
		_prev_position = position_delta
		_prev_rotation = rotation_delta
		_next_position = position_delta
		_next_rotation = rotation_delta
	




extends Object
class_name Util


const PIXEL_TEXTURE := preload("res://core/texture/pixel.png")


## Normaly an angle of 0.0 points right.
## This return the same angle, but 0.0 points up instead.
static func angle_up(angle: float) -> float:
	angle += PI * 0.5
	if angle > PI:
		angle -= TAU
	return angle;


## target in "local space" (target - my_position).
## Return position in the same local space.
## ~4 iterations will give good result.
static func predict_position(
	target: Vector2,
	target_velocity: Vector2,
	speed: float,
	iteration: int) -> Vector2:
	var new_target := target
	for i in iteration:
		var time_to_target := new_target.length() / speed
		new_target = target + target_velocity * time_to_target
	return new_target


## See `predict_position`
## This use global space instead.
static func predict_position_global(
	from: Vector2,
	global_target: Vector2,
	target_velocity: Vector2,
	speed: float,
	iteration: int) -> Vector2:
	return predict_position(
		global_target - from, target_velocity, speed, iteration) + from

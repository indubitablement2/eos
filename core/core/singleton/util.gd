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
static func predict_position(
	target: Vector2,
	target_velocity: Vector2,
	speed: float,
	iteration: int) -> Vector2:
	for i in iteration:
		var time_to_target := target.length() / speed
	return target

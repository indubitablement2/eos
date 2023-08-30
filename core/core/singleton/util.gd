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



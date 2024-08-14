extends CollisionShape2D
class_name HullShield

## global_point is where damage originated.
## multipliers:
## x: hull
## y: armor
## z: shield
## w: emp
func damage(
	amount: float,
	global_point: Vector2,
	damage_multiplier: Vector4) -> void:
	pass

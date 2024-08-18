extends CollisionShape2D
class_name EntityShield

## global_point is where damage originated.
## multipliers:
## x: entity
## y: armor
## z: shield
## w: emp
func damage(
	amount: float,
	global_point: Vector2,
	damage_multiplier: Vector4) -> void:
	pass

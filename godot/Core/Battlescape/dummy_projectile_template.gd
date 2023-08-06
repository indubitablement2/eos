extends DummyProjectile
class_name DummyProjectileTemplate

## Needs to be an instance of Entity.
@export var projectile_scene : PackedScene

## -y is forward.
@export var initial_relative_velocity := Vector2(0.0, -200.0)


func fire(from: Turret) -> void:
	hide()
	
	var e : Entity = projectile_scene.instantiate()
	e.add_collision_exception_with(from.entity)
	
	e.global_transform = global_transform
	
	e.linear_velocity = initial_relative_velocity.rotated(global_rotation)
	
	Battlescape.add_child(e)


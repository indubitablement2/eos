extends Area2D
class_name Bullet


@export_group("Save")
@export
var velocity := Vector2.ZERO
@export
var ttl := 1.0
@export
var damage := 100.0


func _process(delta: float) -> void:
	position += velocity * delta
	ttl -= delta
	if ttl < 0.0:
		queue_free()


func set_team(team: int) -> void:
	# Collide with all hull minus our team.
	collision_mask = Layers.ALL_HULL & ~(Layers.TEAM << Layers.TEAM_OFFSET * team)

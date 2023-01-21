class_name HullData extends Sprite2D

## See EntityData's simulation_script.
@export var simulation_script: Script
## See EntityData's render_script.
@export var render_script: Script

@export_subgroup("Defence")
## Maximum hull hp for the whole hull.
@export var hull := 100
## Maximum armor hp of individual partitions(squares).
##
## Material will be overridden to display damage.
@export var armor := 100

@export_subgroup("Physic")
## Node will not exist in-game.
##
## Only support CollisionPolygon2D(circle, rectangle)
## and CollisionShape2D(only used for the point array).
@export var collision_shape: NodePath
@export var density := 1.0

# Needed to identify hull when scanning parent entity's children.
func _is_hull_data() -> void:
	pass

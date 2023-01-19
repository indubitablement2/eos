class_name HullData extends Sprite2D

## Not implememted yet
@export var hull_script: Script

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
## and CollisionShape2D(only used for the points).
@export var collision_shape: NodePath
@export var density := 1.0

func _is_hull_data() -> bool:
	return true

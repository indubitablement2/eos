class_name EntityData extends Node2D

## Not implememted yet
@export var entity_script: Script

@export_subgroup("Mobility")
@export var linear_acceleration := 32.0
@export var angular_acceleration := 16.0
@export var max_linear_velocity := 256.0
@export var max_angular_velocity := 64.0

func _is_entity_data() -> bool:
	return true

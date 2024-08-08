extends RigidBody2D
class_name Hull


static var HULL_SCENES : Array[PackedScene] = [
	load("res://common/hull/janitor/janitor.tscn"),
]
## Idx into HULL_SCENES.
@export var hull_scene_idx: int

@export var hull_max := 1000.0
var hull_max_multiplier := 1.0
func get_hull_max() -> float:
	return hull_max * hull_max_multiplier

@export var armor_max := 0.0
var armor_max_multiplier := 1.0
func get_armor_max() -> float:
	return armor_max * armor_max_multiplier

@export var linear_acceleration := 100.0
@export var angular_acceleration := 1.0
@export var max_linear_velocity := 100.0
@export var max_angular_velocity := 1.0

@export_range(0.0, PI, 0.001) var rear_arc := 1.0

@export_enum("None","Ship","Seek") var ai := "None"



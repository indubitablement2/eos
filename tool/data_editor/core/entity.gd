@tool
extends Sprite2D
class_name Entity

enum EntityType {
	Ship = 0,
	Missile = 1,
	Fighter = 2,
	Projectile = 3,
}

@export var draw_estimated_radius := true : set = set_draw_estimated_radius
@export var draw_marker := true : set = set_draw_marker
@export var estimated_radius := 100.0 : set = set_estimated_radius

@export var entity_type := EntityType.Ship
## used mostly for missile/fighter/projectile
@export var wish_ignore_same_team := false
@export var force_ignore_same_team := false

@export_category("Mobility")
## In unit/seconds.
@export var linear_acceleration := 4.0
## In radian/seconds.
@export var angular_acceleration := 4.0
## In unit/seconds.
@export var max_linear_velocity := 1.0
## In radian/seconds.
@export var max_angular_velocity := 1.0

func to_json(entity_data_idx: int) -> Dictionary:
	var hulls = null
	for child in get_children():
		if child is Hull:
			hulls = child.to_json()
	
	return {
		"entity_data_idx": entity_data_idx,
		"wish_ignore_same_team": wish_ignore_same_team,
		"force_ignore_same_team": force_ignore_same_team,
		"mobility" : {
			"linear_acceleration": linear_acceleration,
			"angular_acceleration": angular_acceleration,
			"max_linear_velocity": max_linear_velocity,
			"max_angular_velocity": max_angular_velocity,
		},
		"hulls": hulls,
		"entity_type": entity_type
	}

func set_draw_estimated_radius(value: bool) -> void:
	draw_estimated_radius = value
	queue_redraw()

func set_draw_marker(value: bool) -> void:
	draw_marker = value
	queue_redraw()

func set_estimated_radius(value: float) -> void:
	estimated_radius = value
	queue_redraw()

func _draw() -> void:
	if draw_estimated_radius:
		draw_arc(Vector2.ZERO, estimated_radius, 0.0, TAU, 64, Color.BLUE, 3, false)
	if draw_marker:
		draw_set_transform(Vector2.ZERO, PI * 0.25)
		draw_rect(Rect2(-estimated_radius, -estimated_radius, estimated_radius * 2.0, estimated_radius * 2.0), Color.RED, false, 3)




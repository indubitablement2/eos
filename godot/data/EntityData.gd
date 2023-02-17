@tool
class_name EntityData extends Node2D

## Optional script used for the simulation.
## Needs to extends EntityScript and be fully deterministic.
@export var simulation_script: GDScript
## Optional script used for rendering.
## Will replace data script with this one.
## Otherwise data script is simply removed.
@export var render_script: GDScript

@export_subgroup("Mobility")
@export var linear_acceleration := 32.0
@export var angular_acceleration := 16.0
@export var max_linear_velocity := 256.0
@export var max_angular_velocity := 64.0

@export_subgroup("Other")
## An aproximate radius for this entity. Used for rendering.
@export var aproximate_radius := 64.0 : set = set_aproximate_radius
@export var hide_aproximate_radius_preview := false : set = set_hide_aproximate_radius_preview

func _draw() -> void:
	if !hide_aproximate_radius_preview:
		draw_circle(Vector2.ZERO, aproximate_radius, Color(1.0, 1.0, 1.0, 0.1))

func set_aproximate_radius(value: float) -> void:
	aproximate_radius = value
	queue_redraw()

func set_hide_aproximate_radius_preview(value: bool) -> void:
	hide_aproximate_radius_preview = value
	queue_redraw()

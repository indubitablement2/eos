@tool
extends Node2D
class_name Planet


enum Tool {
	NONE,
	RANDOMIZE
}

@export
var tool := Tool.NONE : set = set_tool


@export
var distance: float : set = set_distance

const RADIUS_MIN := 0.5
const RADIUS_MAX := 3.0
@export
var radius : float : set = set_radius


func _draw() -> void:
	draw_arc(
		Vector2.ZERO,
		distance,
		0.0,
		INF,
		64,
		Color.from_hsv(distance / 300.0, 0.5, 1.0, 0.5),
		radius,
		false)


func set_tool(value: Tool) -> void:
	match value:
		Tool.RANDOMIZE:
			radius = randf_range(RADIUS_MIN, RADIUS_MAX)


func set_distance(value: float) -> void:
	distance = value
	queue_redraw()

func set_radius(value: float) -> void:
	radius = value
	queue_redraw()

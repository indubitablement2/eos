@tool
extends Node2D
class_name HullTool

func _physics_process(_delta: float) -> void:
	queue_redraw()

func _draw() -> void:
	var hull := get_parent() as Hull
	if !hull:
		return
	
	draw_line(
		Vector2.ZERO,
		Vector2(100.0, 0.0).rotated(PI - hull.rear_arc),
		Color.RED)
	draw_line(
		Vector2.ZERO,
		Vector2(100.0, 0.0).rotated(PI + hull.rear_arc),
		Color.RED)
	draw_arc(
		Vector2.ZERO,
		100.0,
		PI - hull.rear_arc,
		PI + hull.rear_arc,
		32,
		Color.RED)

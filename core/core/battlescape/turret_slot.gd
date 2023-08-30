@tool
extends Node2D
class_name TurretSlot

enum TurretWeight {
	LIGHT,
	MEDIUM,
	HEAVY,
}
@export
var turret_weight_amx := TurretWeight.LIGHT


## If > PI, can rotate without blocking.
@export_range(0.0, 3.142)
var firing_arc := PI:
	set = set_firing_arc
func set_firing_arc(value: float) -> void:
	firing_arc = value
	if Engine.is_editor_hint():
		queue_redraw()


func _draw() -> void:
	if Engine.is_editor_hint():
		draw_line(
			Vector2.ZERO,
			Vector2(0.0, -100.0),
			Color(1.0, 1.0, 1.0, 0.25),
			-1.0,
			true)
		if firing_arc < PI:
			draw_line(
				Vector2.ZERO,
				Vector2(0.0, -100.0).rotated(-firing_arc),
				Color.ALICE_BLUE,
				-1.0,
				true)
			draw_line(
				Vector2.ZERO,
				Vector2(0.0, -100.0).rotated(firing_arc),
				Color.ALICE_BLUE,
				-1.0,
				true)




@tool
extends Node2D
class_name System


enum Tool {
	NONE,
	RANDOMIZE,
}

@export
var tool := Tool.NONE : set = set_tool

@export
var radius : float : set = set_radius


func _draw() -> void:
	draw_circle(Vector2.ZERO, radius, Color(1.0, 1.0, 1.0, 0.1))


func set_tool(value: Tool) -> void:
	match value:
		Tool.RANDOMIZE:
			for child in get_children():
				remove_child(child)
				child.queue_free()
			
			var planet_distance := randf_range(15.0, 40.0)
			while planet_distance < 120.0:
				var planet := Planet.new()
				add_child(planet)
				planet.set_owner(get_tree().edited_scene_root)
				
				planet.name = str(get_child_count() - 1)
				planet.set_tool(Planet.Tool.RANDOMIZE)
				planet.distance = planet_distance
				
				planet_distance += 5.0 + absf(randfn(8.0, 25.0))
				
				if randf() < 0.3:
					break
			
			# Fit system radius
			radius = get_child(-1).distance + 50.0


func fix_overlapping_planets() -> void:
	while !_fix_overlapping_planets():
		pass

## Return if successful.
func _fix_overlapping_planets() -> bool:
	for child in get_children():
		for other_child in get_children():
			if child == other_child:
				continue
			
			var distance := absf(child.distance - other_child.distance)
			if distance < child.radius + other_child.radius + 1.0:
				child.distance += Planet.RADIUS_MAX
				return false
	return true


func set_radius(value: float) -> void:
	radius = value
	queue_redraw()


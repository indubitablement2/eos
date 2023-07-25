extends CollisionPolygon2D
class_name Hull

## Can either be of type:
## - CollisionPolygon2D
## - CollisionShape2D(circle/rectangle)

@export_category("Defence")
@export var hull := 100
@export var armor := 100

func to_json(idx: int) -> Dictionary:
	return {
		"idx": idx,
		"initial_translation_x": Global.sim_scale(position.x),
		"initial_translation_y": Global.sim_scale(position.y),
		"initial_angle": rotation,
		"defence": {
			"hull": hull,
			"armor": armor,
		},
		"shape": _shape_to_json(),
	}

func _shape_to_json() -> Dictionary:
	var convex := Geometry2D.decompose_polygon_in_convex(polygon)
	assert(!convex.is_empty(), "convex shape has no point")
	
	var convex_hulls := []
	for points in convex:
		var points_x := []
		var points_y := []
		for point in points:
			points_x.push_back(Global.sim_scale(point.x))
			points_y.push_back(Global.sim_scale(point.y))
		convex_hulls.push_back({
			"points_x": points_x,
			"points_y": points_y
		})
	
	return {
		"translation_x": Global.sim_scale(position.x),
		"translation_y": Global.sim_scale(position.y),
		"angle": rotation,
		"compound": convex_hulls
	}




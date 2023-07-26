extends CollisionPolygon2D
class_name Hull

## Can either be of type:
## - CollisionPolygon2D
## - CollisionShape2D(circle/rectangle)
##
## Hull transform is ignored.

@export var density := 1.0
## Set this if you want the hull to be a circle/rectangle instead.
@export var is_basic_shape : CollisionShape2D = null

@export_category("Defence")
@export var hull := 100
@export var armor := 100

func to_json() -> Dictionary:
	return {
		"shape": _shape_to_json(),
		"density": density,
		"defence": {
			"hull": hull,
			"armor": armor,
		},
	}

func _shape_to_json() -> Dictionary:
	if is_basic_shape:
		if is_basic_shape.shape is CircleShape2D:
			return {
				"ball": {
					"radius": Global.scale2sim(is_basic_shape.shape.radius)
				}
			}
		else:
			return {
				"cuboid": {
					"hx": Global.scale2sim(is_basic_shape.shape.size.x),
					"hy": Global.scale2sim(is_basic_shape.shape.size.y)
				}
			}
	else:
		var convex := Geometry2D.decompose_polygon_in_convex(polygon)
		assert(!convex.is_empty(), "convex shape has no point")
		
		var compound := convex.map(Global.packedvec2arr)
		
		return {
			"compound": compound
		}
	




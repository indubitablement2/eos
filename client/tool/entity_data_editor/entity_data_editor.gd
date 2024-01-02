@tool
extends Node2D


@export var data : EntityData

@export_category("Physics")
## Support position, but not rotation or scale.
@export var shape : Node2D
@export var mass_radius := 1.0 :
	set = set_mass_radius
func set_mass_radius(value: float) -> void:
	mass_radius = value
	queue_redraw()
@export var density := 1.0
@export_flags_2d_physics var memberships := 1
@export_flags_2d_physics var filter := 1

@export_category("Server Events")
enum EntityEvent {
	AddAiShip,
	AddAiSeek,
}
@export var on_new : Array[EntityEvent]


func _ready() -> void:
	print(JSON.stringify(entity_data_json(), "\t", false))


var _t := 0.0 
func _process(delta: float) -> void:
	_t += delta
	if _t > 1.0:
		_t = 0.0
		#queue_redraw()


func _draw() -> void:
	draw_set_transform(
		Vector2.ZERO,
		0.0,
		Vector2(Util.RENDER_SCALE, Util.RENDER_SCALE))
	
	var armor_rect := compute_armor_rect()
	armor_rect.size *= Util.ARMOR_CELLS_SIZE
	draw_rect(armor_rect, Color.AQUA, false)
	
	draw_arc(
		Vector2.ZERO,
		mass_radius,
		0.0,
		INF,
		64,
		Color.CHOCOLATE)


## position in server unit, size in armor cell (int)
func compute_armor_rect() -> Rect2:
	var rect := Rect2()
	
	if shape is CollisionPolygon2D:
		var points : PackedVector2Array = shape.polygon
		
		if !points.is_empty():
			rect = Rect2(points[0], Vector2.ZERO)
		
		for p in points:
			rect = rect.expand(p)
		
		rect.position *= Util.SERVER_SCALE
		rect.size *= Util.SERVER_SCALE
	elif shape is CollisionShape2D:
		if shape.shape is CircleShape2D:
			var r : float = shape.shape.radius * Util.SERVER_SCALE
			rect = Rect2(-r, -r, r * 2, r * 2)
		elif shape.shape is RectangleShape2D:
			var s : Vector2 = shape.shape.size * Util.SERVER_SCALE
			rect = Rect2(s * -0.5, s)
	
	if shape:
		rect.position += shape.position * Util.SERVER_SCALE
	
	rect = rect.grow(Util.ARMOR_CELLS_SIZE)
	
	rect.size /= Util.ARMOR_CELLS_SIZE
	rect.size = rect.size.ceil()
	rect.size = rect.size.clamp(Vector2(3, 3), Vector2(INF, INF))
	
	return rect


func entity_data_json() -> Dictionary:
	var json := {}
	
	json["hull"] = data.hull
	
	json["armor_max"] = data.armor_max
	var armor_rect := compute_armor_rect()
	json["armor_cells_translation"] = [armor_rect.position.x, armor_rect.position.y]
	json["armor_cells_size"] = [armor_rect.size.x, armor_rect.size.y]
	json["armor_cells"] = null
	
	var shape_translation := shape.position * Util.SERVER_SCALE
	json["shape_translation"] = [shape_translation.x, shape_translation.y]
	if shape is CollisionPolygon2D:
		var points : PackedVector2Array = shape.polygon
		var vertices :Array[Array]= []
		vertices.resize(points.size())
		for i in points.size():
			var point := points[i] * Util.SERVER_SCALE
			vertices[i] = [point.x, point.y]
		json["shape"] = {
			"Polygon" : {
				"vertices" : vertices
			}
		}
	elif shape is CollisionShape2D:
		if shape.shape is CircleShape2D:
			json["shape"] = {
				"Ball" : {
					"radius" : shape.shape.radius * Util.SERVER_SCALE
				}
			}
		elif shape.shape is RectangleShape2D:
			json["shape"] = {
				"Cuboid" : {
					"hx" : shape.shape.size.x * Util.SERVER_SCALE,
					"hy" : shape.shape.size.y * Util.SERVER_SCALE,
				}
			}
		else:
			assert(false, "only circle and rectangle are supported")
	else:
		assert(false, "shape not set")
	json["mass_radius"] = mass_radius
	json["density"] = density
	json["memberships"] = memberships
	json["filter"] = filter
	
	json["linear_acceleration"] = data.linear_acceleration
	json["angular_acceleration"] = data.angular_acceleration
	json["max_linear_velocity"] = data.max_linear_velocity
	json["max_angular_velocity"] = data.max_angular_velocity
	
	json["on_new"] = on_new.map(func(i): return EntityEvent.find_key(i))
	
	return json

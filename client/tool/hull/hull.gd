@tool
extends Node2D


enum TOOL {None, PrintJSON}
@export var tool : TOOL : set = _set_tool

@export_range(0.0, 50000.0, 1.0, "or_greater") var hull_max := 1000.0
@export_range(0.0, 2000.0, 1.0, "or_greater") var armor_max := 0.0

## Expect a 2 float followed by a string.
## 1: Arc direction angle.
## 2: Arc size. -1..1
## 3: Modifiers name.
## Eg. [2, 0, EngineDamage1_5]:
## When hit from angle 2rad +/- arccos(0), hit engine and take 1.5 damage.
@export var damage_modifier_arcs : Array : set = set_damage_modifier_arcs
func set_damage_modifier_arcs(value: Array) -> void:
	damage_modifier_arcs = value
	queue_redraw()

@export_range(0.0, 1000.0, 1.0, "or_greater") var mass_radius := 100.0
@export_range(0.0, 10.0, 0.002, "or_greater") var density := 1.0
@export_flags_2d_physics var memberships := 0
@export_flags_2d_physics var filter := 0

@export var linear_acceleration := 100.0
@export var angular_acceleration := 1.0
@export var max_linear_velocity := 100.0
@export var max_angular_velocity := 1.0

@export_enum("None","Ship","Seek") var ai := "None"

@export var on_new : Array[String]
@export var on_remove : Array[String]


func _draw() -> void:
	var i := 0
	while i + 2 < damage_modifier_arcs.size():
		var dir := damage_modifier_arcs[i] as float
		var arc := acos(damage_modifier_arcs[i + 1] as float)
		i += 3
		var a := dir - arc
		var b := dir + arc
		draw_arc(
			Vector2.ZERO,
			100.0,
			a,
			b,
			32,
			Color.ALICE_BLUE)
		draw_polyline(
			PackedVector2Array([
				Vector2(100.0, 0.0).rotated(a),
				Vector2.ZERO,
				Vector2(100.0, 0.0).rotated(b)]),
			Color.ALICE_BLUE)

func _set_tool(value: TOOL):
	tool = TOOL.None
	match value:
		TOOL.None:
			return
		TOOL.PrintJSON:
			print(JSON.stringify(to_server_data(), "\t", false))


func to_server_data() -> Dictionary:
	return {
		"hull_max" : hull_max,
		"armor_max" : armor_max,
		"damage_modifier_arcs" : _damage_modifier_arcs_server_data(),
		"shape_position" : [get_node("Collider").position.x, get_node("Collider").position.y],
		"shape_rotation" : get_node("Collider").rotation,
		"shape": _shape_server_data(),
		"mass_radius" : mass_radius,
		"density" : density,
		"memberships" : memberships,
		"filter" : filter,
		"linear_acceleration" : linear_acceleration,
		"angular_acceleration" : angular_acceleration,
		"max_linear_velocity" : max_linear_velocity,
		"max_angular_velocity" : max_angular_velocity,
		"ai" : ai,
		"on_new" : on_new,
		"on_remove" : on_remove,
	}

func _damage_modifier_arcs_server_data() -> Array:
	var ret := []
	
	var i := 0
	while i + 2 < damage_modifier_arcs.size():
		var dir := damage_modifier_arcs[i] as float
		var arc := damage_modifier_arcs[i + 1] as float
		var mod := damage_modifier_arcs[i + 2] as String
		i += 3
		
		ret.push_back({
			"arc_direction" : Vector2.RIGHT.rotated(dir),
			"arc_dot" : arc,
			"modifier" : mod
		})
	
	return ret


func _shape_server_data() -> Dictionary:
	var ret = {}
	
	var collider = get_node("Collider")
	if collider is CollisionPolygon2D:
		var vertices = []
		for point in collider.polygon:
			vertices.push_back([point.x, point.y])
		ret["Polygon"] = {"vertices" : vertices}
	elif collider is CollisionShape2D:
		if collider.shape is CircleShape2D:
			ret["Ball"] = {"radius" : collider.shape.radius}
		else:
			ret["Cuboid"] = {
				"hx" : collider.shape.size.x / 2.0,
				"hy" : collider.shape.size.y / 2.0
			}
	
	return ret






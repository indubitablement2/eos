extends Node


@onready
var detector_space := PhysicsServer2D.space_create()
var detector_query_circle : CircleShape2D
var detector_query : PhysicsShapeQueryParameters2D


## Time since battlescape started.
var time := 0.0


func _ready() -> void:
	detector_query_circle = CircleShape2D.new()
	detector_query = PhysicsShapeQueryParameters2D.new()
	detector_query.shape = detector_query_circle
	detector_query.collide_with_areas = true
	detector_query.collide_with_bodies = false
	
	for i in 2:
		var e = preload("res://base/ship/janitor/janitor.tscn").instantiate()
		e.position = Vector2(randf() * 500.0, randf() * 500.0)
		e.set_team(i)
		add_child(e)
		Player.controlled = e


func _exit_tree() -> void:
	PhysicsServer2D.free_rid(detector_space)


func _process(delta: float) -> void:
	time += delta


func delete() -> void:
	set_process(false)
	
	time = 0.0
	
	for child in get_children():
		child.queue_free()


func hull_area_create(hull: Hull) -> RID:
	var circle := PhysicsServer2D.circle_shape_create()
	PhysicsServer2D.shape_set_data(circle, hull.data.radius)
	
	var area := PhysicsServer2D.area_create()
	PhysicsServer2D.area_add_shape(area, circle)
	PhysicsServer2D.area_set_space(area, detector_space)
	PhysicsServer2D.area_set_collision_layer(area, hull.collision_layer)
	PhysicsServer2D.area_attach_object_instance_id(area, hull.get_instance_id())
	
	return area


func hull_area_query(position: Vector2, radius: float, mask: int) -> Array[Dictionary]:
	detector_query_circle.radius = radius
	detector_query.collision_mask = mask
	detector_query.transform = Transform2D(Vector2.RIGHT, Vector2.DOWN, position)
	return PhysicsServer2D.space_get_direct_state(
		detector_space).intersect_shape(detector_query)

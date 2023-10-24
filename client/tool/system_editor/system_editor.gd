@tool
extends Node


@export
var save_systems := false : set = set_save_systems


func spawn(at: Vector2) -> void:
	var circle := CircleShape2D.new()
	circle.radius = randf_range(120.0, 200.0)
	
	var shape := CollisionShape2D.new()
	shape.shape = circle
	
	var rb := RigidBody2D.new()
	rb.add_child(shape)
	rb.position = at
	add_child(rb)


func set_save_systems(_value) -> void:
	for system in get_children():
		for planet in system.get_children():
			pass

extends Node


func _ready() -> void:
	var editors : Array[Node] = [
		load("res://tool/entity_data_editor/ship_gen/gen.tscn").instantiate()
	]
	
	for e in editors:
		e.entity_data_json()
		e.queue_free()
	
	

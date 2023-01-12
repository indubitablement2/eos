extends Node2D

func _ready() -> void:
	var e := get_node("EntityTest3")
	var b := var_to_bytes_with_objects(e)
	print(b)
	var er := bytes_to_var_with_objects(b) as EntityData
	print(er.entity_data.angular_acceleration)

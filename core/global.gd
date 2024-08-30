extends Object
class_name Global

static var entity_data: Array[EntityData] = []

static func _static_entry() -> void:
	var dirs: Array[String] = ["res://"]
	while !dirs.is_empty():
		var dir: String = dirs.pop_back()
		
		if dir == "res://core/" || dir.begins_with("res://_"):
			continue
		
		for subdir in DirAccess.get_directories_at(dir):
			dirs.push_back(dir + subdir + "/")
		
		for path in DirAccess.get_files_at(dir):
			if !path.ends_with("scn"):
				continue
			var scn := load(dir + path)
			if scn is PackedScene:
				var node = scn.instantiate()
				if node is Entity:
					node.data._verify(scn, node)
					entity_data.push_back(node.data)
				node.free()

static func predict_position(
	pos: Vector2,
	vel: float,
	t_pos: Vector2,
	t_vel: Vector2,
	iter := 3) -> Vector2:
	var ret := t_pos
	for _i in iter:
		var time_to_target := pos.distance_to(ret) / vel
		ret = t_pos + t_vel * time_to_target
	return ret

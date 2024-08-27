extends Object
class_name Global

## String (entity scene path) : ShipData
static var ships := {}

static func get_entity_ship_data(entity_scene_path: String) -> ShipData:
	return ships[entity_scene_path]

static func _static_entry() -> void:
	for dir in DirAccess.get_directories_at("res://"):
		if dir == "core" || dir.begins_with("_"):
			continue
		
		if !DirAccess.dir_exists_absolute(dir + "/ship_data"):
			push_warning(dir + "/ship_data doesn't exist")
			continue
		
		for path in DirAccess.get_files_at(dir + "/ship_data"):
			var ship_data := load(dir + "/ship_data/" + path) as ShipData
			ship_data._verify()
			ships[ship_data.entity_scene.resource_path] = ship_data


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

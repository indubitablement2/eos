extends Object
class_name Global

## String (entity scene path) : ShipData
static var ships := {}

func get_entity_ship_data(entity_scene_path: String) -> ShipData:
	return ships[entity_scene_path]

static func _static_entry() -> void:
	for dir in DirAccess.get_directories_at("res://"):
		if dir == "core" || dir.begins_with("_"):
			continue
		
		if !DirAccess.dir_exists_absolute(dir + "/ship_data"):
			push_warning(dir + "/ship_data doesn't exist")
			continue
		
		for path in DirAccess.get_files_at(dir + "/ship_data"):
			var ship_data := load(path) as ShipData
			ships[ship_data.scene.resource_path] = ship_data



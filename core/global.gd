extends Object
class_name Global

## PackedScene (entity scene) : ShipData
static var ships := {}

static func _static_entry() -> void:
	var mod_datas: Array[ModData] = []
	
	for dir in DirAccess.get_directories_at("res://"):
		if dir == "core" || dir.begins_with("_"):
			continue
		
		if !DirAccess.dir_exists_absolute(dir + "/ship_data"):
			push_warning(dir + "/ship_data doesn't exist")
			continue
		
		for path in DirAccess.get_files_at(dir + "/ship_data"):
			var mod_data := load(path) as ModData
			
			for ship_data in mod_data.ships:
				ships[ship_data.scene] = ship_data

extends Node

@export var export_path := "res://server_data.json"

func _ready() -> void:
	print(JSON.stringify([Global.vec2arr(Vector2(1.1, 0.3007812567)), Vector2(3.456, 4.0)]))
	
	var entities := []
	var entity_idx := 0
	for entity in $Entities.get_children():
		entities.push_back(entity.to_json(entity_idx))
		entity_idx += 1
	
	var data := {
		"entities": entities,
	}
	
	var json := JSON.stringify(data, "\t", false, false)
	var fs := FileAccess.open(export_path, FileAccess.WRITE)
	fs.store_string(json)
	fs.close()
	
	get_tree().quit()


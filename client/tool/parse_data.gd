extends Node


@export_file("*.json") var server_data_export_path

@export_dir var entities_editor_folder


func _ready() -> void:
	var data := {
		"instances" : Data.instances,
		"simulations" : {},
		"entities" : [],
		"first_ship" : Data.first_ship,
	}
	
	for node in SimulationsMap.simulation_nodes_arr:
		data["simulations"][node.simulation_id] = {
			"instance" : node.instance_id
		}
	
	for entity_data in Data.entities_data:
		var file_name := entity_data.resource_path.get_file().get_basename()
		var node = load(entities_editor_folder + "/" + file_name + ".tscn").instantiate()
		data["entities"].push_back(node.entity_data_json())
		node.queue_free()
	
	var json := JSON.stringify(data, "\t", false)
	
	FileAccess.open(server_data_export_path, FileAccess.WRITE).store_string(json)
	#print(json)
	
	get_tree().quit()

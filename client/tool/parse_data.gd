extends Resource


@export_category("Export")
@export_global_file("*.json") var server_data_export_path


@export_category("Data")
@export var simulations : Array[PackedScene] = []
@export var entities : Array[PackedScene] = []
@export var first_ship := 0


func _ready() -> void:
	var data := {
		"instances" : Util.instances,
		"simulations" : {},
		"entities" : {},
		"first_ship" : first_ship,
	}
	
	for packed_scene in entities:
		var node := packed_scene.instantiate()
		data["entities"].push_back(node.entity_data_json())
		node.queue_free()
	
	var json := JSON.stringify(data, "\t", false)
	print(json)

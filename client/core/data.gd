extends Node


@export var entities_data : Array[EntityData] = []

## Entity data id of the ship given to client which do not have any.
@export var first_ship := 0


@export var _instances : JSON
## An instance is a public server which handle multiple simulations.
## { instance id (int) : url (String) }
var instances : Dictionary = {}


func _ready() -> void:
	var entity_data_id := 0
	for entity_data in entities_data:
		entity_data.entity_data_id = entity_data_id
		entity_data_id += 1
	
	for instance_id : String in _instances.data:
		instances[instance_id.to_int()] = _instances.data[instance_id]
	_instances.data = null

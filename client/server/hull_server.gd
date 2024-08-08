extends Node
class_name HullServer

static var _next_network_id := 1
static var _free_network_id: Array[int] = []
## Id to communicate between server and client.
var network_id: int

func _notification(what: int) -> void:
	if what == NOTIFICATION_PREDELETE:
		_free_network_id.push_back(network_id)

func _init() -> void:
	if _free_network_id.is_empty():
		network_id = _next_network_id
		_next_network_id += 1
	else:
		network_id = _free_network_id.pop_back()

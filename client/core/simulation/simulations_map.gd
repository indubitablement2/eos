extends Node2D


## { simulation id (int) : node (SimulationNode) }
var simulation_nodes : Dictionary = {}
var simulation_nodes_arr : Array[SimulationNode] = []


func _ready() -> void:
	for node : SimulationNode in get_children():
		var simulation_id := node.name.to_int()
		node.simulation_id = simulation_id
		simulation_nodes[simulation_id] = node
	
	for node : SimulationNode in simulation_nodes.values():
		simulation_nodes_arr.push_back(node)

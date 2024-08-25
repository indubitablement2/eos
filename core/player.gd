extends Node
class_name Player

static var node: Player

var ships: Array[Entity] = []

static func create() -> void:
	assert(!node)
	node = Player.new()
	Entry.get_parent().add_child(node)

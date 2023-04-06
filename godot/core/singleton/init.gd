extends Node

func _ready() -> void:
	pass

func _exit_tree() -> void:
	Grid.free_memory()
	print("Exiting...")

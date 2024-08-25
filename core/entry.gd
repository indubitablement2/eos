extends Node

func _ready() -> void:
	Global._static_entry()
	Battlescape._static_entry()

func _enter_tree() -> void:
	Battlescape._static_exit()


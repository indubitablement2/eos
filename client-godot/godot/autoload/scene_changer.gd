extends Node
# Manage changing scene.


const MAIN_MENU = 0
const GAME = 1
const EXIT = 2
const NUM_SCENE = 3
var current_scene := -1
onready var current_instance :Node = get_tree().get_root().get_child(get_tree().get_root().get_child_count() - 1)

signal changed_scene(to)

func change_scene(to:int) -> void:
	if to >= NUM_SCENE or to < 0:
		push_error("Tried to change to a non existing scene.")
		return
	elif current_scene == to:
		print_debug("Tried to change scene but it is already current or queued.")
		return
	
	current_scene = to
	call_deferred("_deferred_change_scene", to)


func _deferred_change_scene(to:int) -> void:
	current_instance.free()
	
	match to:
		MAIN_MENU:
			current_instance = load("res://scene/main_menu.tscn").instance()
			print_debug("Changing scene to main_menu.")
		GAME:
			current_instance = load("res://scene/game.tscn").instance()
			print_debug("Changing scene to game.")
		EXIT:
			current_instance = load("res://scene/exit.tscn").instance()
			print_debug("Changing scene to exit.")
	
	get_tree().get_root().add_child(current_instance)
	
	emit_signal("changed_scene", to)

extends Node
class_name JobWander

# requirement
# default priority

# actions
# WANDERING
# goto random nearby spot
# CRAFTING
# bring materials
# goto table
# action
# HUNTING
# find wild animal
# attack
# CUTTING PLANT
# find plant
# goto
# action


func _enter_tree() -> void:
	var pawn := get_parent() as Pawn
	
	while true:
		pawn.queue_path_to(pawn.coordinates + Vector2i(
			randi_range(0, 10),
			randi_range(0, 10)))
		if !await pawn.path_to_finished:
			print("Path canceled")
		else:
			print("Path successful")
		await get_tree().create_timer(10.0, false).timeout



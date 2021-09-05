extends Node2D

var wait_for_update = 60

func _ready() -> void:
	get_node("Chunk").init_generate_chunk(256, 256)
	# Update once to receive render data.
	get_node("Chunk").update()

#func _process(_delta):
#	wait_for_update -= 1
#	if wait_for_update <= 0:
#		wait_for_update = 60
#		get_node("Game").update()

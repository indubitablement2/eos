extends Node


## Time in seconds since the game started.
var time := 0.0


func _process(delta: float) -> void:
	time += delta

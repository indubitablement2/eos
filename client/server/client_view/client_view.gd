extends Area2D


var client_id: int


func _physics_process(_delta: float) -> void:
	for body in get_overlapping_bodies():
		pass

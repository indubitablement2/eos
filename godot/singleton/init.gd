extends Node

func _ready() -> void:
	Engine.register_singleton("Hack", preload("res://singleton/hack.gd"))
	
	queue_free()


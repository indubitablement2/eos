extends Node

func _ready() -> void:
	Engine.register_singleton("Hack", preload("res://singleton/hack.gd"))
	Engine.register_singleton("ClientOOOO", Client.new())


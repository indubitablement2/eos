extends Node2D

func _ready() -> void:
	var n = preload("res://entity test/entity_test_5.tscn").instantiate()
	print(n.get_class())
	print(n.has_method("_is_ship_data"))
	$Client.load_data("res://entity test/entity_test_5.tscn")

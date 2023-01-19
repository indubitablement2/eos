class_name ShipData extends Sprite2D

## Name as shown in-game.
@export var display_name := ""

func _ready() -> void:
	var t = preload("res://debug/error.png")
	print(t, t.get_class())

func _is_ship_data() -> bool:
	return true

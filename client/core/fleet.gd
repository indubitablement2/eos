extends Node2D
class_name Fleet


var previous_position := Vector2.INF
var next_position := Vector2.INF


func _ready() -> void:
	previous_position = next_position
	
	var sp := Sprite2D.new()
	sp.set_texture(preload("res://icon.svg"))
	add_child(sp)


func _process(_delta: float) -> void:
	position = previous_position.lerp(next_position, Metascape.interpolation)

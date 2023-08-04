extends Node2D


const gen := preload("res://Base/Ship/Gen/Gen.tscn")

var label := Label.new()
var e : Array[Entity] = [gen.instantiate(), gen.instantiate()]


func _ready() -> void:
	add_child(label)
	
	e[0].position = Vector2(400.0, 500.0)
	add_child(e[0])
	Player.controlled = e[0]
	
	e[1].position = Vector2(600.0, 500.0)
	add_child(e[1])


func _process(_delta: float) -> void:
	label.text = str(
		e[0].angular_velocity,
		"\n",
		e[1].linear_velocity,
	)

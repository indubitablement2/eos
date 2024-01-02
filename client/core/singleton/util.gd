extends Node

## One server unit is this many Godot units when rendering.
## This is applied globaly using the camera's scale.
const RENDER_SCALE := 256.0
## One godot unit when rendering is this many server units.
## Everything is in server scale by default.
const SERVER_SCALE := 1.0 / RENDER_SCALE

## One armor cell is this many server units.
const ARMOR_CELLS_SIZE := 1.0 / 16.0

## An instance is a public server which handle multiple simulations.
## { instance id (int) : url (String) }
@onready var instances : Dictionary = preload("res://data/instances.json").data


## Time in seconds since the game started.
var time := 0.0


func _process(delta: float) -> void:
	time += delta

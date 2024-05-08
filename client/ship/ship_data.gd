extends Resource
class_name ShipData

## Never remove a ship after is it added.
static var SHIPS : Array[ShipData] = [
	load("res://ship/0.tres"),
]


@export var hull_idx : int
@export_multiline var description : String

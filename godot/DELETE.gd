extends Node2D

#PARENT

func _ready() -> void:
	var receive = $Node2D.call("method")
	print("got: ", receive)
	print($Node2D.v)
	notification(111)

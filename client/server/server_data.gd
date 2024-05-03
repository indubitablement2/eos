extends Resource
class_name ServerData

static var SERVERS : Array[ServerData] = [
	load("res://server/0.tres"),
]

@export var idx : int
@export var ws_addr : String

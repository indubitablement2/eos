extends Node2D

func _ready() -> void:
	GlobalClient.try_load_data("res://entity test/ship_test.tscn")
	var bs := GlobalClient.new_test_battlescape()
	bs.show()
	bs.dbg_print_fleets()
	bs.hash_on_tick(40)
	bs.hash_received.connect(_on_hash_received.bind(40))

func _on_hash_received(bs_hash: int, tick: int) -> void:
	print("Got hash for tick ", tick, ": ", bs_hash)

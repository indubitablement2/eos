extends Node2D

var bs : ClientBattlescape

func _unhandled_input(event: InputEvent) -> void:
	if event.is_action_pressed("target"):
		var eid := bs.get_entity_at(get_global_mouse_position())
		print(eid)
		bs.cmd_control_ship(eid)

func _ready() -> void:
	GlobalClient.load_data()
	bs = GlobalClient.new_test_battlescape()
	bs.show()
	bs.hash_on_tick(40)
	bs.hash_received.connect(_on_hash_received.bind(40), 1)

func _on_hash_received(bs_hash: int, tick: int) -> void:
	print("Got hash for tick ", tick, ": ", bs_hash)
	bs.dbg_print_fleets()

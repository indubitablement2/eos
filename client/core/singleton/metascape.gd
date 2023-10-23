extends Node2D


var local_time := 0.0
var server_time := 0.0
var interpolation_start := 0.0
var interpolation_end := 1.0
var interpolation := 0.0
var time_dilation_curve := preload("res://core/resource/time_dilation_curve.tres")
var moving_avg_time_dif := 0.5
var states : Array[PackedByteArray] = []

## {int: Fleet}
var fleets := {}


@onready
var camera : Camera = $Camera


func _unhandled_input(event: InputEvent) -> void:
	if !event.is_pressed():
		return
	if event is InputEventMouseButton:
		if event.button_index == MOUSE_BUTTON_LEFT:
			send_move_fleet(123, get_global_mouse_position())
			get_viewport().set_input_as_handled()
	elif event is InputEventScreenTouch:
		send_move_fleet(123, get_global_mouse_position())
		get_viewport().set_input_as_handled()


func _process(delta: float) -> void:
	if !ServerConnection.is_logged_in():
		return
	
	local_time += delta
	
	# >0.5: ahead
	# <0.5: behind
	var time_dif :=  local_time - server_time + 0.5
	if time_dif > 1.0:
		# Too far ahead.
		local_time = server_time
		moving_avg_time_dif = 0.5
		print_debug("Hard catch up as " , time_dif - 0.5, " ahead")
	elif time_dif < 0.0:
		# Too far behind.
		local_time = server_time - 0.2
		moving_avg_time_dif = 0.5
		print_debug("Hard catch up as " , absf(time_dif - 0.5), " behind")
	else:
		moving_avg_time_dif = moving_avg_time_dif * 0.95 + time_dif * 0.05
	
	Engine.time_scale = time_dilation_curve.sample_baked(moving_avg_time_dif)
	
	# Consume next state.
	if !states.is_empty() && local_time > interpolation_end:
		interpolation_start = interpolation_end
		interpolation_end = states[0].decode_double(4)
		_apply_state(states.pop_front())
	
	# Compute interpolation delta.
	if interpolation_end < interpolation_start:
		interpolation_end = interpolation_start
	var interpolation_dif := interpolation_end - interpolation_start
	if interpolation_dif < 0.01:
		interpolation = 0.0
	else:
		interpolation = (local_time - interpolation_start) / interpolation_dif


func add_state_packet(packet: PackedByteArray) -> void:
	var time := packet.decode_double(4)
	assert(time > server_time)
	
	server_time = time
	states.push_back(packet)


func send_move_fleet(fleet_id: int, to_position: Vector2) -> void:
	var buf := PackedByteArray()
	buf.resize(20)
	buf.encode_u32(0, 0)
	buf.encode_u64(4, fleet_id)
	buf.encode_float(12, to_position.x)
	buf.encode_float(16, to_position.y)
	ServerConnection.socket.send(buf, WebSocketPeer.WRITE_MODE_BINARY)


func _apply_state(state: PackedByteArray) -> void:
	var num_partial_fleets_info := state.decode_u32(12)
	var num_full_fleets_info := state.decode_u32(16)
	var num_state := state.decode_u32(20)
	var num_remove_fleet := state.decode_u32(24)
	var i := 28
	
	var new_fleet : Array[Fleet] = []
	for _i in num_partial_fleets_info:
		var id := state.decode_u64(i)
		i += 8
		var num_ship := state.decode_u32(i)
		i += 4
		
		var fleet : Fleet
		if !fleets.has(id):
			fleet = Fleet.new()
			add_child(fleet)
			fleets[id] = fleet
			new_fleet.push_back(fleet)
		else:
			fleet = fleets[id]
		
		fleet.set_partial_info(num_ship)
	
	for _i in num_full_fleets_info:
		var _id := state.decode_u64(i)
		i += 8
		var _data := state.decode_u32(i)
		i += 4
	
	for _i in num_state:
		var id := state.decode_u64(i)
		i += 8
		
		var next_position := Vector2(
			state.decode_float(i),
			state.decode_float(i + 4))
		i += 8
		
		var fleet : Fleet = fleets[id]
		fleet.previous_position = fleet.next_position
		fleet.next_position = next_position
	
	for _i in num_remove_fleet:
		var id := state.decode_u64(i)
		i += 8
		
		fleets[id].queue_free()
		fleets.erase(id)
	
	for fleet in new_fleet:
		fleet.previous_position = fleet.next_position

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


func _apply_state(state: PackedByteArray) -> void:
	var num_add_fleet := state.decode_u32(12)
	var num_state := state.decode_u32(16)
	var num_remove_fleet := state.decode_u32(20)
	var i := 24
	
	# Add fleets.
	for _i in num_add_fleet:
		var id := state.decode_u64(i)
		i += 8
		
		var fleet := Fleet.new()
		add_child(fleet)
		fleets[id] = fleet
	
	# Set fleets state.
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
	
	# Remove fleets.
	for _i in num_remove_fleet:
		var id := state.decode_u64(i)
		i += 8
		
		fleets[id].queue_free()
		fleets.erase(id)

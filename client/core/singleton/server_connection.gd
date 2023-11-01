extends Node


enum LoginState {
	NONE,
	SENDING_LOGIN,
	WAITING_ON_RESPONSE,
	SUCCESS,
}

const CENTRAL_ADDR := "ws://[::1]:8461"
const INSTANCE_PORT := 7245

const LOGIN_TIMEOUT := 10.0


var login_timer := 0.0


var central_socket := WebSocketPeer.new()
var central_login_state := LoginState.NONE

var username = null
var password = null


var instance_socket := WebSocketPeer.new()
var instance_login_state := LoginState.NONE
var instance_login_timer := 0.0

## To authenticate ourself to the instance servers.
var token := 0


signal central_disconnected(reason)


func _process(delta: float) -> void:
	central_socket.poll()
	instance_socket.poll()
	
	match central_login_state:
		LoginState.SENDING_LOGIN:
			if central_socket.get_ready_state() == WebSocketPeer.STATE_OPEN:
				central_socket.send_text(JSON.stringify({
					"username": username,
					"password": password}))
				central_login_state = LoginState.WAITING_ON_RESPONSE
			else:
				_login_timeout(delta)
		LoginState.WAITING_ON_RESPONSE:
			if central_socket.get_ready_state() == WebSocketPeer.STATE_CLOSED:
				central_disconnected.emit(central_socket.get_close_reason())
				central_login_state = LoginState.NONE
			else:
				if central_socket.get_available_packet_count() > 0:
					_parse_central_login_response(central_socket.get_packet())
				else:
					_login_timeout(delta)
		LoginState.SUCCESS:
			if central_socket.get_ready_state() == WebSocketPeer.STATE_CLOSED:
				central_disconnected.emit(central_socket.get_close_reason())
				central_login_state = LoginState.NONE
			else:
				while central_socket.get_available_packet_count() > 0:
					_parse_packet()


func log_in() -> void:
	assert(central_login_state == LoginState.NONE)
	
	var err := central_socket.connect_to_url(CENTRAL_ADDR, TLSOptions.client_unsafe())
	assert(err == 0)
	
	central_login_state = LoginState.SENDING_LOGIN
	login_timer = 0.0


func is_logged_in() -> bool:
	return central_login_state == LoginState.SUCCESS


func central_send_move_fleet(
	metascape_id: int,
	fleet_id: int,
	to: Vector2) -> void:
		if central_login_state != LoginState.SUCCESS:
			push_warning("Not connected")
			return
		
		var packet := PackedByteArray()
		packet.resize(24)
		
		packet.encode_u32(0, 0)
		packet.encode_u64(4, metascape_id)
		packet.encode_u64(12, fleet_id)
		packet.encode_float(16, to.x)
		packet.encode_float(20, to.y)
		
		central_socket.send(packet)

func central_send_create_practice_battlescape() -> void:
		if central_login_state != LoginState.SUCCESS:
			push_warning("Not connected")
			return
		
		var packet := PackedByteArray()
		packet.resize(4)
		
		packet.encode_u32(0, 1)
		
		central_socket.send(packet)

func instance_send_spawn_entity(
	entity_data_id: int,
	translation: Vector2,
	angle: float) -> void:
		if central_login_state != LoginState.SUCCESS:
			push_warning("Not connected")
			return
		
		var packet := PackedByteArray()
		packet.resize(20)
		
		packet.encode_u32(0, 0)
		packet.encode_u32(4, entity_data_id)
		packet.encode_float(8, translation.x)
		packet.encode_float(12, translation.y)
		packet.encode_float(16, angle)
		
		central_socket.send(packet)


func _parse_central_login_response(packet: PackedByteArray) -> void:
	token = packet.decode_u64(4)
	var success := packet.decode_u32(12) != 0
	
	if success:
		central_login_state = LoginState.SUCCESS
	else:
		central_login_state = LoginState.NONE


func _login_timeout(delta: float) -> void:
	login_timer += delta
	if login_timer > LOGIN_TIMEOUT:
		_login_failed("Timeout")

func _login_failed(reason: String) -> void:
	push_warning("Login failed: ", reason)
	central_login_state = LoginState.NONE
	login_timer = 0.0

func _parse_packet() -> void:
	var packet := central_socket.get_packet()
	if central_socket.was_string_packet():
		var text := packet.get_string_from_utf8()
		print(text)
	else:
		var packet_type := packet.decode_u32(0)
		match packet_type:
			0:
				Metascape.add_state_packet(packet)
			_:
				assert(false)

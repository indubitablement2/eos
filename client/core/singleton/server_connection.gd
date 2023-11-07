extends Node


enum LoginState {
	Closed,
	InProgress,
	LoggedIn,
}


## Id for packet received from central server.
enum CentralClientPacket {
	LoginSuccess = 1,
	ChangeBattlescape = 5,
	GlobalMessage = 10,
}
## Id for packet sent to central server. 
enum ClientCentralPacket {
	JoinBattlescape = 5,
	GlobalMessage = 10,
}

const CENTRAL_ADDR := "ws://[::1]:8461"
var central_socket := WebSocketPeer.new()
## Is filled only during login attempt.
var central_login_packet := []
var central_logged_in := LoginState.Closed

## Our unique id.
var client_id := 0
## To authenticate ourself to the instance servers.
var token := 0


var instance_socket := WebSocketPeer.new()
var instance_logged_in := LoginState.Closed


signal central_login_result(success: bool, reason: String)
signal central_disconected(reason: String)


func _process(_delta: float) -> void:
	central_socket.poll()
	instance_socket.poll()
	
	while central_socket.get_available_packet_count() > 0:
		_parse_central_packet()
	
	match central_socket.get_ready_state():
		WebSocketPeer.STATE_OPEN:
			if central_logged_in == LoginState.Closed:
				central_socket.put_var(central_login_packet)
				central_logged_in = LoginState.InProgress
		WebSocketPeer.STATE_CLOSED:
			if central_logged_in == LoginState.InProgress:
				central_login_result.emit(false, central_socket.get_close_reason())
			elif central_logged_in == LoginState.LoggedIn:
				central_disconected.emit(central_socket.get_close_reason())
			central_logged_in = LoginState.Closed
	
	while instance_socket.get_available_packet_count() > 0:
		pass
	
	match instance_socket.get_ready_state():
		WebSocketPeer.STATE_OPEN:
			if instance_logged_in == LoginState.Closed:
				instance_socket.put_var([client_id, token])
				# We asume success.
				instance_logged_in = LoginState.LoggedIn
		WebSocketPeer.STATE_CLOSED:
			if instance_logged_in != LoginState.Closed:
				push_warning("Instance closed: ", instance_socket.get_close_reason())
			instance_logged_in = LoginState.Closed


func log_in(username: String, password: String) -> void:
	assert(central_socket.get_ready_state() == WebSocketPeer.STATE_CLOSED)
	
	var error := central_socket.connect_to_url(
		CENTRAL_ADDR,
		TLSOptions.client_unsafe())
	if error == Error.OK:
		central_login_packet = [true, username, password]
	else:
		central_login_result.emit(false, error_string(error))

func _log_in_instance(url: String) -> void:
	instance_socket.connect_to_url(
		url,
		TLSOptions.client_unsafe())


## Send a message in global chat.
func send_central_global_message(
	channel: int,
	message: String) -> void:
	central_socket.put_var([
		ClientCentralPacket.GlobalMessage,
		channel,
		message])

## Ask to join a battlescape.
func send_central_join_battlescape(battlescape_id: int) -> void:
	central_socket.put_var([
		ClientCentralPacket.JoinBattlescape,
		battlescape_id])


func _parse_central_packet() -> void:
	var packet : Array = central_socket.get_var()
	print(packet)
	
	var packet_type : int = packet[0]
	match packet_type:
		CentralClientPacket.LoginSuccess:
			client_id = packet[1]
			token = packet[2]
			central_logged_in = LoginState.LoggedIn
			central_login_result.emit(true, "Login successful")
		CentralClientPacket.ChangeBattlescape:
			var battlescape_id : int = packet[1]
			Battlescape.battlescape_id = battlescape_id
			if battlescape_id == 0:
				instance_socket.close()
			elif !packet[2]:
				instance_socket.close()
				_log_in_instance("ws://" + packet[3])
		CentralClientPacket.GlobalMessage:
			print(packet[3])
		_:
			push_error("Unknow packet: ", packet)
			assert(false)

func _parse_instance_packet() -> void:
	var packet : Array = central_socket.get_var()
	print(packet)
	
#	var packet_type : int = packet[0]
#	match packet_type:

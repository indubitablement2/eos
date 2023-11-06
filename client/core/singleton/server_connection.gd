extends Node


## Id for packet received from central server.
enum CentralClientPacket {
	LoginSuccess = 1,
	GlobalMessage = 10,
}
## Id for packet sent to central server. 
enum ClientCentralPacket {
	GlobalMessage = 10,
}

const CENTRAL_ADDR := "ws://[::1]:8461"
var central_socket := WebSocketPeer.new()
var central_login_packet := []
var central_logged_in := false

## Our unique id.
var client_id := 0
## To authenticate ourself to the instance servers.
var token := 0


var instance_socket := WebSocketPeer.new()


signal central_login_result(success: bool, reason: String)
signal central_disconected(reason: String)


func _process(_delta: float) -> void:
	central_socket.poll()
	instance_socket.poll()
	
	while central_socket.get_available_packet_count() > 0:
		_parse_central_packet()
	
	match central_socket.get_ready_state():
		WebSocketPeer.STATE_OPEN:
			if !central_login_packet.is_empty():
				central_socket.put_var(central_login_packet)
				central_login_packet = []
		WebSocketPeer.STATE_CLOSING:
			if central_logged_in:
				central_disconected.emit(central_socket.get_close_reason())
			else:
				central_login_result.emit(false, central_socket.get_close_reason())
			central_logged_in = false
		WebSocketPeer.STATE_CLOSED:
			if !central_login_packet.is_empty():
				central_login_packet = []
				central_login_result.emit(false, "Server unreachable")
	
	while instance_socket.get_available_packet_count() > 0:
		pass


func log_in(username: String, password: String) -> void:
	assert(central_socket.get_ready_state() == WebSocketPeer.STATE_CLOSED)
	
	var error := central_socket.connect_to_url(
		CENTRAL_ADDR,
		TLSOptions.client_unsafe())
	if error == Error.OK:
		central_login_packet = [true, username, password]
	else:
		central_login_result.emit(false, error_string(error))


## Send a message in global chat.
func send_central_global_message(
	channel: int,
	message: String) -> void:
	central_socket.put_var([
		ClientCentralPacket.GlobalMessage,
		channel,
		message])


func _parse_central_packet() -> void:
	var packet : Array = central_socket.get_var()
	var packet_type : int = packet[0]
	match packet_type:
		CentralClientPacket.LoginSuccess:
			client_id = packet[1]
			token = packet[2]
			central_logged_in = true
			central_login_result.emit(true, "Login successful")
		CentralClientPacket.GlobalMessage:
			print(packet[3])
		_:
			push_error("Unknow packet: ", packet)
			assert(false)

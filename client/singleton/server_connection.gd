extends Node

var socket := WebSocketPeer.new()
var udp_peer : PacketPeerUDP
const SERVER_ADDR = "[::1]:6134"

enum ConnectionToServerState {
	Unconnected,
	SendingFirstPacket,
	ReceivingFirstPacket,
	Connected
}
var connection_state := ConnectionToServerState.Unconnected

signal connected_to_server_result(success: bool)
signal disconnected_from_server

func _process(_delta: float) -> void:
	socket.poll()
	
	match socket.get_ready_state():
		WebSocketPeer.STATE_OPEN:
			match connection_state:
				ConnectionToServerState.SendingFirstPacket:
					_send_first_packet()
				ConnectionToServerState.ReceivingFirstPacket:
					_receive_first_packet()
				ConnectionToServerState.Connected:
					_handle_packets()
		WebSocketPeer.STATE_CLOSED:
			match connection_state:
				ConnectionToServerState.SendingFirstPacket:
					connected_to_server_result.emit(false)
				ConnectionToServerState.ReceivingFirstPacket:
					connected_to_server_result.emit(false)
				ConnectionToServerState.Connected:
					disconnected_from_server.emit()
			
			connection_state = ConnectionToServerState.Unconnected

func connect_to_server() -> void:
	assert(connection_state == ConnectionToServerState.Unconnected)
	
	var result := socket.connect_to_url(SERVER_ADDR)
	if result == OK:
		connection_state = ConnectionToServerState.SendingFirstPacket
	else:
		connected_to_server_result.emit(false)
		print("Connect to server failed: ", result)

func _send_first_packet() -> void:
	connection_state = ConnectionToServerState.ReceivingFirstPacket
	
	var wish_udp = 0
	if !OS.has_feature("wasm"):
		socket.set_no_delay(false)
		
		udp_peer = PacketPeerUDP.new()
		udp_peer.bind(0)
		wish_udp = udp_peer.get_local_port()
	
	socket.send_text(JSON.stringify({
		"wish_udp": wish_udp
	}))

func _receive_first_packet() -> void:
	if socket.get_available_packet_count():
		var p : Dictionary = JSON.parse_string(
			socket.get_packet().get_string_from_utf8())
		
		print(p)
		
		var server_udp_port : int = p["server_udp_port"]
		if server_udp_port != 0:
			udp_peer.connect_to_host("", server_udp_port)
		
		connection_state = ConnectionToServerState.Connected
		connected_to_server_result.emit(true)

func _handle_packets() -> void:
	if udp_peer:
		while udp_peer.get_available_packet_count():
			_handle_packet(udp_peer.get_packet())
	while socket.get_available_packet_count():
		_handle_packet(socket.get_packet())

func _handle_packet(p: PackedByteArray) -> void:
	var cursor := 1
	
	match p[0]:
		0:
			pass






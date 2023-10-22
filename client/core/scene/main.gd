extends Node

var socket := WebSocketPeer.new()
var logged := false

func _ready() -> void:
	var result := socket.connect_to_url("[::1]:8461", TLSOptions.client_unsafe())
	print(result)


func _process(_delta: float) -> void:
	socket.poll()
	
	$Label.set_text(str(socket.get_ready_state()))
	
	if !logged && socket.get_ready_state() == WebSocketPeer.STATE_OPEN:
		socket.send_text(JSON.stringify({
			"username": "Banana",
			"password": "Bob"}))
		logged = true
	
	while socket.get_available_packet_count() > 0:
		var packet := socket.get_packet()
		if socket.was_string_packet():
			var text := packet.get_string_from_utf8()
			print(text)
		else:
			print(packet, "\n")

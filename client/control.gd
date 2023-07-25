extends Control

var socket := WebSocketPeer.new()

func _ready() -> void:
	var result := socket.connect_to_url("127.0.0.1:2345")
	print("Connected to server: ", result)

func _process(_delta: float) -> void:
	socket.poll()
	if socket.get_ready_state() == WebSocketPeer.STATE_OPEN:
		while socket.get_available_packet_count():
			var p := socket.get_packet()
			print(p.get_string_from_utf8())

func _on_button_pressed() -> void:
	if socket.get_ready_state() == WebSocketPeer.STATE_OPEN:
		var result := socket.send_text($TextEdit.text)
		print("Sent message to server: ", result)

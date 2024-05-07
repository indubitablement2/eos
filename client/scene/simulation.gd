extends Node2D
class_name Simulation

static var node : Simulation

static var username : String = "I_am_a_test"
static var password : String = "12345678"
static var client_id := 0

var ws := WebSocketPeer.new()
## 0 -> waiting to connect then send login
## 1 -> waiting for login response
## 2 -> ready
var _state := 0

func _ready() -> void:
	node = self
	
	var err := ws.connect_to_url(ServerData.SERVERS[0].ws_addr)
	if err:
		print(error_string(err))
	_state = 0

func _process(_delta: float) -> void:
	ws.poll()
	
	if ws.get_ready_state() == WebSocketPeer.STATE_CLOSED:
		push_warning(ws.get_close_code())
		queue_free()
	
	if ws.get_ready_state() != WebSocketPeer.STATE_OPEN:
		return
	
	match _state:
		0:
			Postcard.start_encode()
			Postcard.put_bool(false)
			Postcard.put_string(username)
			Postcard.put_string(password)
			Postcard.put_bool(true)
			postcard_send()
			_state = 1
		1:
			if ws.get_available_packet_count() > 0:
				Postcard.start_decode(ws.get_packet())
				client_id = Postcard.get_u64()
				_state = 2
		2:
			while ws.get_available_packet_count() > 0:
				print(ws.get_packet())


func postcard_send() -> void:
	ws.send(Postcard.finish_encode(), WebSocketPeer.WRITE_MODE_BINARY)

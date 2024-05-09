extends Node2D
class_name Simulation

static var node : Simulation

const DT := 0.1

static var username : String = "I_am_a_test"
static var password : String = "12345678"
static var client_id := 0

var ws := WebSocketPeer.new()
## 0 -> waiting to connect then send login
## 1 -> waiting for login response
## 2 -> ready
var _ws_state := 0

var hulls : Array[Hull] = []
## Delta toward next state.
var sim_dt := 0.0
var _states : Array[PackedByteArray] = []

func _ready() -> void:
	node = self
	
	var err := ws.connect_to_url(ServerData.SERVERS[0].ws_addr)
	if err:
		print(error_string(err))
	_ws_state = 0

func _process(delta: float) -> void:
	ws.poll()
	
	if ws.get_ready_state() == WebSocketPeer.STATE_CLOSED:
		#push_warning(ws.get_close_code())
		#queue_free()
		return
	
	if ws.get_ready_state() != WebSocketPeer.STATE_OPEN:
		return
	
	match _ws_state:
		0:
			Postcard.start_encode()
			Postcard.put_bool(false)
			Postcard.put_string(username)
			Postcard.put_string(password)
			Postcard.put_bool(true)
			postcard_send()
			_ws_state = 1
		1:
			if ws.get_available_packet_count() > 0:
				Postcard.start_decode(ws.get_packet())
				client_id = Postcard.get_u64()
				_ws_state = 2
		2:
			while ws.get_available_packet_count() > 0:
				_handle_packet(ws.get_packet())
	
	sim_dt += delta / DT
	if sim_dt > 1.0:
		if _states.is_empty():
			sim_dt = 1.0
			push_warning("out of state")
		else:
			_apply_state()
			sim_dt -= 1.0
			if sim_dt > 0.5:
				push_warning("too fast")
				sim_dt = 0.5


func _handle_packet(packet: PackedByteArray) -> void:
	if packet[0] == 0:
		_states.push_back(packet)
		return
	
	Postcard.start_decode(packet)
	match Postcard.get_u64():
		0: # State (already handled)
			pass

func _apply_state() -> bool:
	if _states.is_empty():
		return false
	
	Postcard.start_decode(_states.pop_front())
	
	Postcard.get_u64()
	var global_bitfield := Postcard.get_u8()
	var resync := global_bitfield & 0b1 != 0
	
	var hull_idx := 0
	while Postcard.get_remaining_bytes() > 0:
		var bitfield := Postcard.get_u8()
		if bitfield & 0b1: # Remove
			hulls[hull_idx].queue_free()
			hulls.remove_at(hull_idx)
			continue
		if bitfield & 0b10: # Is new
			var hull : Hull = Hull.HULL_SCENES[Postcard.get_u64()].instantiate()
			hull.hull_id = Postcard.get_u64()
			add_child(hull)
			hulls.insert(hull_idx, hull)
		
		hulls[hull_idx].apply_state(
			resync,
			Vector2(Postcard.get_vector2i()) / 8.0,
			float(Postcard.get_i64()) * PI / 512.0
		)
		
		hull_idx += 1
	
	return true

func postcard_send() -> void:
	ws.send(Postcard.finish_encode(), WebSocketPeer.WRITE_MODE_BINARY)

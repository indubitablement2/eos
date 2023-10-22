extends Node

enum LoginState {
	NONE,
	SENDING_LOGIN,
	WAITING_ON_RESPONSE,
	SUCCESS,
}

const LOGIN_TIMEOUT := 10.0

var socket := WebSocketPeer.new()
var login_state := LoginState.NONE
var login_timer := 0.0

var username = null
var password = null

signal login_result(fail_reason)
signal server_disconnected(reason)

func _ready() -> void:
	login()

func _process(delta: float) -> void:
	socket.poll()
	
	$Label.set_text(str(socket.get_ready_state()))
	
	match login_state:
		LoginState.SENDING_LOGIN:
			if socket.get_ready_state() == WebSocketPeer.STATE_OPEN:
				socket.send_text(JSON.stringify({
					"username": username,
					"password": password}))
				login_state = LoginState.WAITING_ON_RESPONSE
			else:
				_login_timeout(delta)
		LoginState.WAITING_ON_RESPONSE:
			if socket.get_available_packet_count() > 0:
				var response = JSON.parse_string(
					socket.get_packet().get_string_from_utf8())
				if response["success"]:
					login_state = LoginState.SUCCESS
					login_result.emit(null)
				else:
					_login_failed(response["reason"])
			else:
				_login_timeout(delta)
		LoginState.SUCCESS:
			if socket.get_ready_state() == WebSocketPeer.STATE_CLOSED:
				server_disconnected.emit(socket.get_close_reason())
				login_state = LoginState.NONE
			else:
				while socket.get_available_packet_count() > 0:
					_parse_packet()

func login() -> void:
	assert(login_state == LoginState.NONE)
	
	var err := socket.connect_to_url("[::1]:8461", TLSOptions.client_unsafe())
	assert(err == 0)
	
	login_state = LoginState.SENDING_LOGIN
	login_timer = 0.0

func _login_timeout(delta: float) -> void:
	login_timer += delta
	if login_timer > LOGIN_TIMEOUT:
		_login_failed("Timeout")

func _login_failed(reason: String) -> void:
	push_warning("Login failed: ", reason)
	login_state = LoginState.NONE
	login_timer = 0.0
	login_result.emit(reason)

func _parse_packet() -> void:
	var packet := socket.get_packet()
	if socket.was_string_packet():
		var text := packet.get_string_from_utf8()
		print(text)
	else:
		print(packet)

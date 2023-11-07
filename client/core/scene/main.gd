extends Node

func _ready() -> void:
	ServerConnection.central_login_result.connect(_on_central_login_result)
	ServerConnection.central_disconected.connect(_on_central_disconnect)
	
	ServerConnection.log_in("user", "pass")


func _on_central_login_result(result: bool, reason: String) -> void:
	if result:
		print("is work!")
		ServerConnection.send_central_global_message(123, "Hello server!")
		ServerConnection.send_central_join_battlescape(444)
	else:
		push_warning("don't work: ", reason)

func _on_central_disconnect(reason: String) -> void:
	print("central gone: ", reason)

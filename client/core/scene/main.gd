extends Node

func _ready() -> void:
	ServerConnection.central_login_result.connect(_on_central_login_result)
	
	ServerConnection.log_in("user", "pass")


func _on_central_login_result(result: bool, reason: String) -> void:
	if result:
		print("is work!")
		ServerConnection.send_central_global_message(123, "Hello server!")
	else:
		push_warning("don't work: ", reason)

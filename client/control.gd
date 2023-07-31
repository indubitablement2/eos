extends Control

func _on_button_pressed() -> void:
	ServerConnection.connect_to_server()

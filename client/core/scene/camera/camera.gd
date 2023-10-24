extends Camera2D
class_name Camera


var free_look := true
var dragging := false


func _unhandled_input(event: InputEvent) -> void:
	if free_look:
		if event.is_action(&"camera_drag"):
			dragging = event.is_pressed()
			get_viewport().set_input_as_handled()
		
		if event.is_action(&"camera_zoom_in"):
			zoom /= ProjectSettings.get_setting_with_override(
				&"game_setting/camera/zoom_speed")
			get_viewport().set_input_as_handled()
		if event.is_action(&"camera_zoom_out"):
			zoom *= ProjectSettings.get_setting_with_override(
				&"game_setting/camera/zoom_speed")
			get_viewport().set_input_as_handled()
		
		if event is InputEventMouseMotion && dragging:
			position -= event.relative / zoom.x
			get_viewport().set_input_as_handled()


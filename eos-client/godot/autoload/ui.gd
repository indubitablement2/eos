extends CanvasLayer


onready var root_control := Control.new()


func _ready() -> void:
	root_control.set_anchors_preset(Control.PRESET_WIDE)
	root_control.set_mouse_filter(Control.MOUSE_FILTER_IGNORE)
	add_child(root_control)
	# Add chat.
	root_control.add_child(preload("res://component/chat.tscn").instance())


func toggle_visible_ui(toggle:bool = false) -> void:
	root_control.set_visible(toggle)

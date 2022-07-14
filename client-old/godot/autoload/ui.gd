extends CanvasLayer


onready var root_control := Control.new()


func _ready() -> void:
	var err := 0
	err += GlobalVariable.connect("cheated", self, "_on_cheated")
	assert(err == OK)
	
	
	root_control.set_anchors_preset(Control.PRESET_WIDE)
	root_control.set_mouse_filter(Control.MOUSE_FILTER_IGNORE)
	add_child(root_control)
	# Add chat.
	root_control.add_child(preload("res://component/chat.tscn").instance())


func toggle_visible_ui(toggle:bool = false) -> void:
	root_control.set_visible(toggle)


func _on_cheated() -> void:
	# Add water mark.
	var tr = TextureRect.new()
	tr.set_texture(load("res://texture/ui/SkullCrossed_64.png"))
	tr.set_modulate(Color(0.0, 0.0, 0.0, 0.1))
#	tr.set_anchors_and_margins_preset(Control.PRESET_BOTTOM_RIGHT,Control.PRESET_MODE_KEEP_SIZE)
	root_control.add_child(tr)
	tr.set_anchors_and_margins_preset(Control.PRESET_BOTTOM_RIGHT)
	
#	tr.set_anchors_preset(Control.PRESET_BOTTOM_RIGHT)
	

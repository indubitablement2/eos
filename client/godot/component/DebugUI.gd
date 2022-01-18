extends HBoxContainer

func _process(_delta: float) -> void:
	if is_visible():
		for c in get_children():
			c.update()

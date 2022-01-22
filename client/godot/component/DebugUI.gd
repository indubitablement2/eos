extends HBoxContainer

var d := 0.0

func _process(delta: float) -> void:
	if is_visible():
		d += delta
		if d > 0.1:
			d = 0.0
			for c in get_children():
				c.update()

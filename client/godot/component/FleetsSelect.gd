extends HBoxContainer

var wish_control := -1

func _ready() -> void:
	add_user_signal("WishControlChanged", [{"name": "index", "type": typeof(0) }])

func set_num_fleet(num: int) -> void:
	# Clear the previous buttons.
	for child in get_children():
		child.queue_free()
	
	if num <= 0:
		return
	
	# Add new button.
	for i in num:
		var b := Button.new()
		b.set_text(str(i))
		b.set_toggle_mode(true)
		b.connect("pressed", self, "_on_fleet_select", [i])
		add_child(b)
	
	if wish_control >= num:
		# Fleet no more exist.
		wish_control = -1
	
	# Reselect fleet.
	_set_wish_control(wish_control)

func _set_wish_control(index: int) -> void:
	wish_control = index
	
	var i = 0
	for child in get_children():
		child.set_pressed(i == wish_control)
		i += 1

func _on_fleet_select(num: int) -> void:
	if num == wish_control:
		# Wish to remove control.
		num = -1
	
	_set_wish_control(num)
	emit_signal("WishControlChanged", wish_control)

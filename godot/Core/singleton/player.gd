extends Node


var controlled : Entity = null : set = set_controlled

var aim_at_cursor := false


func _unhandled_input(event: InputEvent) -> void:
	if event.is_action("aim_at_cursor"):
		aim_at_cursor = event.is_pressed()
		get_viewport().set_input_as_handled()


func _physics_process(_delta: float) -> void:
	if controlled:
		var mouse_pos := controlled.get_global_mouse_position()
		
		var wish_dir := Vector2(
			Input.get_action_strength("right")
				- Input.get_action_strength("left"),
			Input.get_action_strength("down")
				- Input.get_action_strength("up"))
		
		if aim_at_cursor:
			# Drone control
			controlled.wish_angvel_aim(mouse_pos)
			controlled.wish_linvel_absolute(wish_dir)
		else:
			# Tank control
			controlled.wish_angvel_force(wish_dir.x)
			controlled.wish_linvel_relative(Vector2(0.0, wish_dir.y))


func set_controlled(value: Entity) -> void:
	if controlled:
		controlled.player_controlled = false
		controlled.tree_exiting.disconnect(_on_controlled_tree_exiting)
	
	controlled = value
	
	if controlled:
		controlled.player_controlled = true
		controlled.tree_exiting.connect(_on_controlled_tree_exiting)


func _on_controlled_tree_exiting() -> void:
	controlled.player_controlled = false
	controlled = null

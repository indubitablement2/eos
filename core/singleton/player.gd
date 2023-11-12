extends Node2D


var mouse_position := Vector2.ZERO
var actions := 0


var controlled : Hull = null:
	set = set_controlled
func set_controlled(value: Hull) -> void:
	if controlled != null:
		controlled.tree_exiting.disconnect(_on_controlled_tree_exiting)
		controlled.player_controlled = false
	controlled = value
	if controlled != null:
		controlled.tree_exiting.connect(
			_on_controlled_tree_exiting, CONNECT_ONE_SHOT)
		controlled.player_controlled = true


func _process(_delta: float) -> void:
	mouse_position = get_global_mouse_position()
	
	if controlled != null:
		var dir := Vector2(
			Input.get_action_strength("right") - Input.get_action_strength("left"),
			Input.get_action_strength("down") - Input.get_action_strength("up"))
		var strafe := (Input.get_action_strength("strafe_right")
			- Input.get_action_strength("strafe_left"))
		
		if Input.is_action_pressed("aim_at_cursor"):
			# Drone control.
			controlled.wish_angular_velocity_aim_smooth(mouse_position)
			dir.x += strafe
			controlled.wish_linear_velocity_force_absolute(dir.limit_length(1.0))
		else:
			# Tank control.
			controlled.wish_angular_velocity_force(dir.x)
			controlled.wish_linear_velocity_force_relative(Vector2(strafe, dir.y))
		
		actions = 0
		if Input.is_action_pressed("primary"):
			actions |= 1
		if Input.is_action_pressed("secondary"):
			actions |= 2
		# 4, 8, ...
		
		if Input.is_action_just_pressed("shield"):
			push_error("todo shield")


func _on_controlled_tree_exiting() -> void:
	controlled = null

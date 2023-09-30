extends Node



#func _physics_process(_delta: float) -> void:
#	if controlled:
#		var mouse_pos := controlled.get_global_mouse_position()
#
#		var wish_dir := Vector2(
#			Input.get_action_strength("right")
#				- Input.get_action_strength("left"),
#			Input.get_action_strength("down")
#				- Input.get_action_strength("up"))
#
#		if Input.is_action_pressed("aim_at_cursor"):
#			# Drone control
#			controlled.wish_angvel_aim(mouse_pos)
#			controlled.wish_linvel_absolute(wish_dir)
#		else:
#			# Tank control
#			controlled.wish_angvel_force(wish_dir.x)
#			controlled.wish_linvel_relative(Vector2(0.0, wish_dir.y))
#
#		controlled.aim_at = mouse_pos
#
#		controlled.actions = 0
#		if Input.is_action_pressed("fire_primary"):
#			controlled.actions |= TurretSlot.TurretGroup.PRIMARY << 14
#			if Input.is_action_just_pressed("fire_primary"):
#				controlled.actions |= TurretSlot.TurretGroup.PRIMARY
#		if Input.is_action_pressed("fire_secondary"):
#			controlled.actions |= TurretSlot.TurretGroup.SECONDARY << 14
#			if Input.is_action_just_pressed("fire_secondary"):
#				controlled.actions |= TurretSlot.TurretGroup.SECONDARY
#
#
#func set_controlled(value: Entity) -> void:
#	if controlled:
#		controlled.player_controlled = false
#		controlled.tree_exiting.disconnect(_on_controlled_tree_exiting)
#
#	controlled = value
#
#	if controlled:
#		controlled.player_controlled = true
#		controlled.tree_exiting.connect(_on_controlled_tree_exiting)
#
#
#func _on_controlled_tree_exiting() -> void:
#	controlled.player_controlled = false
#	controlled = null

extends Node2D
class_name BattlescapePlayer

static var unlock_aim := false
static var hold_time := 0.2

var entity: Entity = null:
	set = set_entity
func set_entity(value: Entity) -> void:
	if entity:
		entity.get_ship_ai().player_controlled = false
		entity.tree_exiting.disconnect(_on_entity_tree_exiting)
	entity = value
	if entity:
		entity.get_ship_ai().player_controlled = true
		entity.tree_exiting.connect(_on_entity_tree_exiting, CONNECT_ONE_SHOT)
func _on_entity_tree_exiting() -> void:
	entity.get_ship_ai().player_controlled = false
	entity = null

var _control_ship_hold_timer := -1.0

func _init() -> void:
	process_priority = -2

func _unhandled_input(event: InputEvent) -> void:
	if event.is_action_pressed("unlock_aim"):
		unlock_aim = !unlock_aim
	elif event.is_action("control_ship"):
		if event.is_pressed():
			_control_ship_hold_timer = hold_time
		else:
			if _control_ship_hold_timer > 0.0:
				_control_ship_hold_timer = -1.0
				_try_control_ship()

func _physics_process(_delta: float) -> void:
	if !entity:
		return
	
	var dir := Vector2(
		Input.get_action_strength("right") - Input.get_action_strength("left"),
		Input.get_action_strength("backward") - Input.get_action_strength("forward"))
	
	var ai := entity.get_ship_ai()
	if ai.is_auto_pilot:
		if dir.is_zero_approx():
			return
		else:
			ai.is_auto_pilot = false
	
	# a1: left mouse, rt
	# a2: right mouse, lt
	# shield: f, lb
	# ability: g, rb
	# strategic view: tab, y
	# toggle pd: v, b
	# vent: hold toggle pd
	# control ship: c, a
	# auto pilot: hold spectate
	# target: r, x
	# ship retreat: hold target
	# unlock aim: z, ???
	
	var mouse_pos := get_global_mouse_position()
	
	if unlock_aim:
		# Tank control.
		entity.wish_angular_velocity_force(dir.x)
		entity.wish_linear_velocity_force_relative(Vector2(0.0, dir.y))
	else:
		# Drone control.
		entity.wish_angular_velocity_aim_smooth(mouse_pos)
		entity.wish_linear_velocity_force_absolute(dir.limit_length(1.0))

func _process(delta: float) -> void:
	if _control_ship_hold_timer > 0.0:
		_control_ship_hold_timer -= delta / Battlescape.get_time_scale()
		if _control_ship_hold_timer < 0.0 && entity:
			entity.get_ship_ai().is_auto_pilot = true

func _try_control_ship() -> void:
	var query_pos := get_global_mouse_position()
	var exclude: Array[RID]
	if entity:
		exclude = [entity.get_rid()]
	else:
		exclude = []
	var result := Battlescape.intersect_circle(
		query_pos,
		200.0,
		Battlescape.make_collision_mask(0, Battlescape.COLLISION_FRIEND_SHIP),
		exclude)
	
	var closest: Entity = null
	var closest_dist := INF
	
	for dic in result:
		var collider: Entity = dic["collider"]
		if collider.is_ally:
			continue
		var dist := collider.position.distance_squared_to(query_pos)
		if dist < closest_dist:
			closest = collider
			closest_dist = dist
	
	if closest:
		set_entity(closest)


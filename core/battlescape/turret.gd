extends Sprite2D
class_name Turret


@export
var data : TurretData


@export_group("Save")
@export
var ammo := 0

var ammo_replenish_delay_remaining : float

@export
var fire_delay_remaining := 0.0


@export
var player_action_group := 0


var target : Hull = null:
	set = set_target
func set_target(value: Hull) -> void:
	if target:
		target.tree_exiting.disconnect(_on_target_tree_exiting)
	target = value
	if target:
		target.tree_exiting.connect(_on_target_tree_exiting, CONNECT_ONE_SHOT)
func _on_target_tree_exiting() -> void:
	target = null


var turret_slot : TurretSlot
var hull : Hull

# TODO: stored on the area's shape's filter.
# smalls
# smalls -> target -> ships
# target -> ships
# public bool PointDefence;

const TARGET_QUERY_COOLDOWN := 0.4
var query_cooldown := 0.0


func _enter_tree() -> void:
	# Turret should always be a child of Hull/TurretSlot.
	turret_slot = get_parent()
	hull = turret_slot.get_parent()
	
	hull.turrets.push_back(self)
	
	ammo_replenish_delay_remaining = data.ammo_replenish_delay * hull.ammo_replenish_delay


func _exit_tree() -> void:
	set_target(null)
	hull.turrets.erase(self)


func _process(delta: float) -> void:
	var scaled_delta := delta * hull.time_scale
	
	query_cooldown -=  scaled_delta
	
	var is_player_controlled := hull.player_controlled && player_action_group != 0
	
	# Find where to aim at.
	var aim_at := Vector2.INF
	var wish_fire := false
	if is_player_controlled:
		aim_at = Player.mouse_position
		wish_fire = Player.actions & player_action_group != 0
	else:
		if target:
			# Check that target is still in range.
			if can_target(target):
				# Predict position.
				if data.projectile_speed != INF:
					aim_at = Util.predict_position_global(
						hull.position,
						target.position,
						target.linear_velocity,
						500.0,
						mini(data.prediction_iter + hull.prediction_iter, 3))
				else:
					aim_at = target.position
			else:
				set_target(null)
		elif ammo > 0:
			# Prefer taking target from parent hull. 
			if hull.target:
				if hull.target.data.is_ship() == data.target_ship:
					# Check that target is in range.
					if can_target(hull.target):
						set_target(hull.target)
			
			if target == null:
				_find_target()
	
	# Rotate toward aim at.
	var wish_angle_change := -rotation
	if aim_at.x != INF:
		var angle_to_target := Util.angle_up(get_angle_to(aim_at))
		wish_angle_change = angle_to_target
		if !is_player_controlled:
			wish_fire = absf(angle_to_target) <= data.effective_angle
	var rotation_speed_delta := data.rotation_speed * hull.rotation_speed * scaled_delta
	if turret_slot.firing_arc < PI:
		if absf(rotation + wish_angle_change) > PI:
			wish_angle_change -= signf(wish_angle_change) * TAU
		rotation += clampf(
			wish_angle_change, -rotation_speed_delta, rotation_speed_delta)
		rotation = clampf(
			rotation, -turret_slot.firing_arc, turret_slot.firing_arc);
	else:
		rotation += clampf(
			wish_angle_change, -rotation_speed_delta, rotation_speed_delta)
	
	# Replenish ammo.
	var ammo_max := int(float(data.ammo_max) * hull.ammo_max[data.turret_type])
	var ammo_replenish_delay := data.ammo_replenish_delay * hull.ammo_replenish_delay
	if ammo < ammo_max:
		ammo_replenish_delay_remaining -= scaled_delta
		if ammo_replenish_delay_remaining < 0.0:
			ammo_replenish_delay_remaining += ammo_replenish_delay
			ammo = mini(ammo + data.ammo_replenish_amount , ammo_max)
	else:
		ammo_replenish_delay_remaining = ammo_replenish_delay
	
	# Fire.
	fire_delay_remaining -= scaled_delta
	if wish_fire:
		#TODO: If player controlled, make a sound when out of ammo.
		while ammo > 0 && fire_delay_remaining <= 0.0:
			fire()
			ammo -= 1
			fire_delay_remaining += data.fire_delay * hull.fire_delay
	fire_delay_remaining = maxf(fire_delay_remaining, 0.0)


func fire() -> void:
	push_error("fire should be overwritten")


func effective_range() -> float:
	return data.effective_range * hull.turret_range[data.turret_type]


## Return if other is in range and this turret can rotate toward other.
func can_target(other: Hull) -> bool:
	var r := effective_range() + other.data.radius
	if global_position.distance_squared_to(other.position) < r * r:
		return can_look_at(other)
	else:
		return false

## Return if this turret can rotate toward other.
func can_look_at(other: Hull) -> bool:
	return absf(Util.angle_up(turret_slot.get_angle_to(
		other.position))) < turret_slot.firing_arc + data.effective_angle


func _find_target() -> void:
	if query_cooldown > 0.0:
		return 
	
	var mask := Layers.ALL_HULL_SMALL
	if data.target_ship:
		mask = Layers.ALL_HULL_SHIP
	mask &= ~(Layers.TEAM << Layers.TEAM_OFFSET * hull.team)
	
	#for d in Battlescape.hull_area_query(global_position, effective_range(), mask):
		#var other : Hull = d["collider"]
		#if can_look_at(other):
			#set_target(other)
			#return
	
	# Only add cooldown when failing to find target.
	query_cooldown = TARGET_QUERY_COOLDOWN



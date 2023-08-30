extends Sprite2D
class_name Turret


@export_range(0.0, 20.0, 0.1, "or_greater")
var rotation_speed := 4.0

@export_range(0.0, 10.0, 0.01, "or_greater")
var fire_delay := 0.5

@export_range(1, 200, 1, "or_greater")
var ammo_max := 1000000
@export_range(0.1, 10.0, 0.1, "or_greater")
var ammo_replenish_delay := INF
@export_range(1, 10, 1, "or_greater")
var ammo_replenish_amount := 1


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

const TARGET_QUERY_COOLDOWN := 0.2
var last_target_query := 0.0


func _enter_tree() -> void:
	# Turret should always be a child of Hull/TurretSlot.
	turret_slot = get_parent()
	hull = turret_slot.get_parent()
	
	ammo_replenish_delay_remaining = ammo_replenish_delay


func _exit_tree() -> void:
	set_target(null)


func _process(delta: float) -> void:
	var scaled_delta := delta * hull.time_scale
	
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
	var rotation_speed_delta := rotation_speed * scaled_delta
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
	if ammo < ammo_max:
		ammo_replenish_delay_remaining -= scaled_delta
		if ammo_replenish_delay_remaining < 0.0:
			ammo_replenish_delay_remaining += ammo_replenish_delay
			ammo = mini(ammo + ammo_replenish_amount, ammo_max)
	else:
		ammo_replenish_delay_remaining = ammo_replenish_delay
	
	# Fire.
	fire_delay_remaining -= scaled_delta
	if wish_fire:
		#TODO: If player controlled, make a sound when out of ammo.
		while ammo > 0 && fire_delay_remaining <= 0.0:
			fire()
			ammo -= 1
			fire_delay_remaining += fire_delay
	fire_delay_remaining = maxf(fire_delay_remaining, 0.0)


func fire() -> void:
	push_error("fire should be overwritten")

## Meant to be overwritten to account for hull multiplier.
func effective_range() -> float:
	return data.effective_range


## Return if other is in range and this turret can rotate toward other.
func can_target(other: Hull) -> bool:
	var r := effective_range() + target.data.radius
	if global_position.distance_squared_to(target.position) < r * r:
		return can_look_at(other)
	else:
		return false

## Return if this turret can rotate toward other.
func can_look_at(other: Hull) -> bool:
	return absf(Util.angle_up(turret_slot.get_angle_to(
		other.position))) < turret_slot.firing_arc + data.effective_angle


func _find_target() -> void:
	if (Battlescape.time - last_target_query) < TARGET_QUERY_COOLDOWN:
		return 
	last_target_query = Battlescape.time
	
	var mask := Layers.ALL_HULL_SMALL
	if data.target_ship:
		mask = Layers.ALL_HULL_SHIP
	mask &= ~(Layers.TEAM << Layers.TEAM_OFFSET * hull.team)
	
	for d in Battlescape.hull_area_query(global_position, effective_range(), mask):
		var other : Hull = d["collider"]
		if can_look_at(other):
			set_target(other)
			return


func _verify() -> void:
	ammo = ammo_max





extends Node
class_name ShipAi


@onready
var hull : Hull = get_parent()

var primary_turret : Turret
const FIND_PRIMARY_TURRET_COOLDOWN := 0.5
var find_primary_turret_cooldown := 0.5


func _process(delta: float) -> void:
	if hull.player_controlled:
		return
	
	hull.auto_turret_disabled = false
	
	if hull.target:
		if primary_turret:
			if primary_turret.ammo <= 0:
				if primary_turret.data.ammo_replenish_delay > 20.0:
					primary_turret = null
		else:
			find_primary_turret_cooldown -= delta
			if find_primary_turret_cooldown < 0.0:
				find_primary_turret_cooldown = FIND_PRIMARY_TURRET_COOLDOWN
				find_primary_turret()
		
		var turret_range := 800.0
		if primary_turret:
			turret_range = primary_turret.effective_range()
		
		hull.wish_angular_velocity_aim_smooth(hull.target.position)
		
		var to_target := hull.target.position - hull.position
		var target_dist := to_target.length()
		hull.wish_linear_velocity_position_smooth(
			(to_target / target_dist) * (target_dist - turret_range) + hull.position)
	else:
		# Find a target.
		var closest : Hull = null
		var closest_dist_squared := INF
		for r in Battlescape.hull_area_query(
			hull.position,
			4000.0,
			Layers.ALL_HULL_SHIP & ~(Layers.TEAM << Layers.TEAM_OFFSET * hull.team)):
				var other : Hull = r["collider"]
				var dist := hull.position.distance_squared_to(other.position)
				if dist < closest_dist_squared:
					closest_dist_squared = dist
					closest = other
		
		hull.target = closest


func go_to(pos: Vector2) -> void:
	hull.wish_linear_velocity_position_overshoot(pos)


func find_primary_turret() -> void:
	var primary : Turret = null
	var primary_value := -INF
	for turret in hull.turrets:
		if turret.ammo <= 0 && primary_turret.data.ammo_replenish_delay > 20.0:
			continue
		
		var v := float(turret.player_action_group == 1) * 0.5
		v += float(turret.data.weight)
		v += float(turret.data.turret_type != TurretData.TurretType.MISSILE)
		
		if v > primary_value:
			primary_value = v
			primary = turret
	
	primary_turret = primary





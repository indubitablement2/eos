extends Node2D
class_name Turret


@export var data : TurretData

var ammo := 0
var ammo_replenish_cooldown := 0.0
var fire_cooldown := 0.0

var target : Entity = null : set = set_target
var target_pos := Vector2.ZERO

var turret_slot : TurretSlot
var entity: Entity


func _enter_tree() -> void:
	# Turret should always be a child of Entity/TurretSlot
	turret_slot = get_parent()
	entity = turret_slot.get_parent()
	
	ammo_replenish_cooldown = get_ammo_replenish_delay()
	ammo = get_max_ammo()


func _physics_process(delta: float) -> void:
	var aim_at := Vector2.INF
	if turret_slot.turret_group == 0:
		if target:
			aim_at = target.position
		else:
			# TODO: find target
			pass
	else:
		aim_at = entity.aim_at
	
	var wish_angle_change := -rotation
	if aim_at.x != INF:
		wish_angle_change = get_angle_to(aim_at)
		wish_angle_change += PI * 0.5
		if (wish_angle_change > PI):
			wish_angle_change -= TAU
	
	var rot_speed := get_rotation_speed() * delta
	
	if turret_slot.firing_arc < PI:
		if absf(rotation + wish_angle_change) > PI:
			wish_angle_change -= signf(wish_angle_change) * TAU
		
		rotation += clampf(wish_angle_change, -rot_speed, rot_speed)
		
		rotation = clampf(
			rotation, -turret_slot.firing_arc, turret_slot.firing_arc)
	else:
		rotation += clampf(wish_angle_change, -rot_speed, rot_speed)
	
	var max_ammo := get_max_ammo()
	if ammo < max_ammo:
		ammo_replenish_cooldown -= delta
		if ammo_replenish_cooldown < 0.0:
			ammo_replenish_cooldown = get_ammo_replenish_delay()
			ammo = mini(ammo + get_ammo_replenish_amount(), max_ammo)
	
	var fire_group := int(turret_slot.turret_group)
	if data.auto_fire:
		fire_group <<= 14
	var wish_fire := fire_group & entity.actions != 0
	if fire_group == 0:
		if target:
			wish_fire = (wish_fire 
				|| absf(get_angle_to(target_pos))
				< data.auto_fire_angle_threshold)
	
	fire_cooldown -= delta
	
	if wish_fire:
		# TODO: If player controlled, make a sound when out of ammo
		# and trying to fire.
		
		while ammo > 0 && fire_cooldown <= 0.0:
			fire()
			ammo -= 1
			fire_cooldown += get_fire_delay()
	
	fire_cooldown = maxf(fire_cooldown, 0.0)


func get_rotation_speed() -> float:
	return data.rotation_speed

func get_ammo_replenish_delay() -> float:
	return data.ammo_replenish_delay

func get_ammo_replenish_amount() -> int:
	return data.ammo_replenish_amount

func get_max_ammo() -> int:
	return data.max_ammo

func get_fire_delay() -> float:
	return data.fire_delay


func set_target(value: Entity) -> void:
	if target:
		target.tree_exiting.disconnect(_on_target_tree_exiting)
	
	target = value
	
	if target:
		target.tree_exiting.connect(_on_target_tree_exiting)


func fire() -> void:
	push_error("Base turret shouldn't be called")


func _on_target_tree_exiting() -> void:
	target = null

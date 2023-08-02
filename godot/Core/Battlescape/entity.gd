extends RigidBody2D
class_name Entity


enum EntityCollisionType {
	SHIP = 1 | 2 | 3,
	BANANA = 1 | 5 | 3,
}

enum WishAngvelType {
	NONE,
	STOP,
	AIM,
	ROTATION,
	FORCE
}


@export var hull_srpite : HullSprite
@export var turrets : Array[Turret]
@export var entity_collision_type := EntityCollisionType.SHIP

@export_category("Movement")

@export var linacc_base := 1000.0
var linacc_add := 0.0
var linacc_mul := 1.0

@export var linacc_max_base := 500.0
var linacc_max_add := 0.0
var linacc_max_mul := 1.0

@export var angacc_base := 20.0
var angacc_add := 0.0
var angacc_mul := 1.0

@export var angacc_max_base := 10.0
var angacc_max_add := 0.0
var angacc_max_mul := 1.0


## Infinity used as a flag to not try to change linvel unless above max.
var wish_linvel := Vector2.INF

var wish_angvel_type := WishAngvelType.NONE
var wish_angvel := Vector2.ZERO

var player_controlled := false


func _integrate_forces(state: PhysicsDirectBodyState2D) -> void:
	# Contacts
	var contact_count := state.get_contact_count()
	var contact_idx := 0
	while contact_idx < contact_count:
		var contact_impulse := state.get_contact_impulse(contact_idx)
		print(contact_impulse)
	
	# Integrate angular velocity.
	var angacc := get_angacc() * state.step
	match wish_angvel_type:
		WishAngvelType.NONE:
			if absf(state.angular_velocity) > get_angvel_max():
				state.angular_velocity -= clampf(
					state.angular_velocity,
					-angacc,
					angacc)
		WishAngvelType.STOP:
			state.angular_velocity -= clampf(
					state.angular_velocity,
					-angacc,
					angacc)
		WishAngvelType.AIM:
			state.angular_velocity += _wish_angvel_change(
				get_angle_to(wish_angvel),
				state.angular_velocity,
				angacc,
				get_angvel_max())
		WishAngvelType.ROTATION:
			state.angular_velocity += _wish_angvel_change(
				_shortest_angle(rotation, wish_angvel.x),
				state.angular_velocity,
				angacc,
				get_angvel_max())
		WishAngvelType.FORCE:
			state.angular_velocity += wish_angvel.x * angacc
	
	# Integrate linear velocity.
	var linvel_max := get_linvel_max()
	if (wish_linvel.x != INF 
	|| state.linear_velocity.length_squared() > linvel_max * linvel_max):
		state.linear_velocity += (wish_linvel
		- state.linear_velocity).limit_length(get_linacc() * state.step)


func get_linacc() -> float:
	return (linacc_base + linacc_add) * linacc_mul

func get_linvel_max() -> float:
	return (linacc_max_base + linacc_max_add) * linacc_max_mul

func get_angacc() -> float:
	return (angacc_base + angacc_add) * angacc_mul

func get_angvel_max() -> float:
	return (angacc_max_base + angacc_max_add) * angacc_max_mul


func wish_linvel_none() -> void:
	wish_linvel = Vector2.INF

func wish_linvel_stop() -> void:
	wish_linvel = Vector2.ZERO

func wish_linvel_pos(pos: Vector2) -> void:
	wish_linvel = (pos - position).limit_length(get_linvel_max())

func wish_linvel_pos_overshoot(pos: Vector2) -> void:
	wish_linvel = (pos - position).normalized() * get_linvel_max()

func wish_linvel_absolute_force(force: Vector2) -> void:
	wish_linvel = force.limit_length(1.0) * get_linvel_max()

func wish_linvel_relative_force(force: Vector2) -> void:
	wish_linvel = force.rotated(rotation).limit_length(1.0) * get_linvel_max()


func wish_angvel_none() -> void:
	wish_angvel_type = WishAngvelType.NONE

func wish_angvel_stop() -> void:
	wish_angvel_type = WishAngvelType.STOP

func wish_angvel_aim(pos: Vector2) -> void:
	wish_angvel_type = WishAngvelType.AIM
	wish_angvel = pos

func wish_angvel_rotation(rot: float) -> void:
	wish_angvel_type = WishAngvelType.ROTATION
	wish_angvel.x = rot

func wish_angvel_force(force: float) -> void:
	wish_angvel_type = WishAngvelType.FORCE
	wish_angvel.x = force


func _wish_angvel_change(
	wish_angle_change: float,
	angvel: float,
	angacc: float,
	angvel_max: float
	) -> float:
	var wish_dir := signf(wish_angle_change)
	
	# Reduce target vel when close to target.
	var close_smooth := minf(absf(wish_angle_change), 0.2) / 0.2
	close_smooth *= close_smooth * close_smooth
	
	if wish_dir != signf(angvel):
		return clampf(angvel_max * wish_dir * close_smooth, -angacc, angacc)
	
	var time_to_target := absf(wish_angle_change / angvel)
	var time_to_stop := absf(angvel / (angacc * 60.0))
	
	if time_to_target > time_to_stop:
		return clampf(angvel_max * wish_dir * close_smooth, -angacc, angacc)
	else:
		# Going too fast. Risk of overshoot, so we slow down.
		return clampf(angvel_max * -wish_dir * close_smooth, -angacc, angacc)

func _shortest_angle(a: float, b: float) -> float:
	var diff := b - a
	diff = fmod(diff + PI, TAU) - PI
	return diff

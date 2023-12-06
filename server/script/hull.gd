extends RigidBody2D
class_name HullServer


const ARMOR_CELL_OFFSET : Array[Vector2i] = [
	Vector2i(-1, -1),
	Vector2i(0, -1),
	Vector2i(1, -1),
	Vector2i(-1, 0),
	Vector2i(0, 0),
	Vector2i(1, 0),
	Vector2i(-1, 1),
	Vector2i(0, 1),
	Vector2i(1, 1)]
const ARMOR_CELL_EFFECT : PackedFloat32Array = [
	0.041059,
	0.126094,
	0.041059,
	0.126094,
	0.331388,
	0.126094,
	0.041059,
	0.126094,
	0.041059]
## This many pixel equal 1 armor cell.
const ARMOR_SCALE = 16.0
## There is always at least this many armor cell.
const ARMOR_SIZE_MIN = Vector2i(3, 3)
## Used to not go oob
const ARMOR_CENTER_MIN = Vector2i(1, 1)
## Used to not go oob
const ARMOR_CENTER_MAX = Vector2i(2, 2)


## Shared between hulls. Do not modify at runtime.
@export var base_data : HullData
## Unique to this instance.
@export var data : HullData


@export_group("Wish Movement")
enum WishAngularVelocityType
{
	## Keep current angular velocity.
	NONE,
	## Keep current angular velocity.
	## Do nothing unless above max, then slow down until back to max.
	KEEP,
	## Try to reach 0 angular velocity.
	STOP,
	## Set angular velocity to aim at a point in global space without overshoot.
	AIM_SMOOTH,
	## Same as AIM_SMOOTH, but always try to go at max velocity.
	## Faster to integrate than AIM_SMOOTH
	## and makes almost no difference for low angular acceleration.
	AIM_OVERSHOOT,
	## Rotate left or right [-1..1]. Force should be clamped to 1.
	FORCE,
}
@export var wish_angular_velocity_type := WishAngularVelocityType.NONE
@export var wish_angular_velocity := Vector2.ZERO

enum WishLinearVeloctyType {
	## Keep current linear velocity.
	## Do nothing.
	NONE,
	## Keep current linear velocity.
	## Do nothing unless above max, then slow down until back to max.
	KEEP,
	## Try to reach 0 linear velocity.
	STOP,
	## Cancel our current velocity to reach position as fast as possible.
	## Does not overshoot.
	POSITION_SMOOTH,
	## Same as POSITION_SMOOTH, but always try to go at max velocity.
	POSITION_OVERSHOOT,
	## Force toward an absolute direction. -y is up.
	## Magnitude bellow 1 can be used to accelerate slower.
	## Magnitude should be clamped to 1.
	FORCE_ABSOLUTE,
	## Force toward a direction relative to current rotation. 
	## x is forward, y is right.
	## Magnitude bellow 1 can be used to accelerate slower.
	## Magnitude should be clamped to 1.
	FORCE_RELATIVE,
}
@export var wish_linear_velocity_type := WishLinearVeloctyType.NONE
@export var wish_linear_velocity := Vector2.ZERO


## null when no target.
var target: HullServer = null:
	set = set_target
func set_target(value: HullServer) -> void:
	if target:
		target.tree_exiting.disconnect(_on_target_tree_exiting)
	target = value
	if target:
		target.tree_exiting.connect(_on_target_tree_exiting, CONNECT_ONE_SHOT)
func _on_target_tree_exiting() -> void:
	target = null

var auto_turret_disabled := false
var player_controlled := false


var turrets : Array[Turret] = []


var detector : RID


signal took_damage(amount: Vector4, cell_center: Vector2i)


# TODO: Add detector
#func _enter_tree() -> void:
	#detector = Battlescape.hull_area_create(self)

# TODO: Remove detector
#func _exit_tree() -> void:
	#PhysicsServer2D.free_rid(detector)

# TODO: Move detector
#func _physics_process(_delta: float) -> void:
	#PhysicsServer2D.area_set_transform(detector, transform)


func _integrate_forces(state: PhysicsDirectBodyState2D) -> void:
	# Contacts.
	# TODO: Contact signal
	for i in state.get_contact_count():
		var contact_impulse := state.get_contact_impulse(i)
		var contact_magnitude := contact_impulse.length_squared()
		if contact_magnitude > 400.0:
			var amount := sqrt(contact_magnitude)
			damage(
				Vector4(amount, amount, amount, amount),
				state.get_contact_local_position(i))
	
	# Angular velocity.
	match wish_angular_velocity_type:
		WishAngularVelocityType.KEEP:
			if absf(state.angular_velocity) > data.angular_velocity_max:
				_integrate_angvel(
					clampf(state.angular_velocity, -data.angular_velocity_max, data.angular_velocity_max),
					state)
		WishAngularVelocityType.STOP:
			if !is_zero_approx(state.angular_velocity):
				_integrate_angvel(0.0, state)
		WishAngularVelocityType.AIM_SMOOTH:
			var offset := get_angle_to(wish_angular_velocity)
			var wish_dir := signf(offset)
			var close_smooth := minf(absf(offset), 0.2) / 0.2
			close_smooth *= close_smooth * close_smooth

			if wish_dir == signf(state.angular_velocity):
				var time_to_target := absf(offset / state.angular_velocity)
				var time_to_stop := absf(
					state.angular_velocity / (data.angular_acceleration))
				if (time_to_target < time_to_stop):
					close_smooth *= -1.0

			_integrate_angvel(
				wish_dir * data.angular_velocity_max * close_smooth,
				state)
		WishAngularVelocityType.AIM_OVERSHOOT:
			var wish_dir := signf(get_angle_to(wish_angular_velocity))
			_integrate_angvel(wish_dir * data.angular_velocity_max, state)
		WishAngularVelocityType.FORCE:
			_integrate_angvel(
				wish_angular_velocity.x * data.angular_velocity_max,
				state)

	# Linear velocity.
	match wish_linear_velocity_type:
		WishLinearVeloctyType.KEEP:
			if state.linear_velocity.length() > data.linear_velocity_max:
				_integrate_linvel(
					state.linear_velocity.limit_length(data.linear_velocity_max),
					state)
		WishLinearVeloctyType.STOP:
			_integrate_linvel(Vector2.ZERO, state)
		WishLinearVeloctyType.POSITION_SMOOTH:
			var to_position := wish_linear_velocity - position
			if to_position.length_squared() < 100.0:
				# We are on target.
				_integrate_linvel(Vector2.ZERO, state)
			else:
				_integrate_linvel(
					to_position.limit_length(data.linear_velocity_max),
					state)
		WishLinearVeloctyType.POSITION_OVERSHOOT:
			var to_position := wish_linear_velocity - position
			if to_position.is_zero_approx():
				to_position = Vector2(0.0, data.linear_velocity_max)
			else:
				to_position = to_position.normalized() * data.linear_velocity_max
			_integrate_linvel(to_position, state)
		WishLinearVeloctyType.FORCE_ABSOLUTE:
			_integrate_linvel(wish_linear_velocity * data.linear_velocity_max,
			state)
		WishLinearVeloctyType.FORCE_RELATIVE:
			_integrate_linvel(
				wish_linear_velocity.rotated(rotation) * data.linear_velocity_max,
				state)


## point is where damage originated in global space.
## amount:
## x: hull
## y: armor
## z: shield
## w: emp
func damage(amount: Vector4, point: Vector2) -> void:
	# Find nearest armor cell.
	var cell_center := Vector2i(to_local(point) - data.offset / ARMOR_SCALE).clamp(
		ARMOR_CENTER_MIN,
		data.armor_relative_image.get_size() - ARMOR_CENTER_MAX)

	# Compute armor effectiveness from nearby cells.
	var armor_effect := 0.0
	var i := 0
	for offset in ARMOR_CELL_OFFSET:
		armor_effect += data.armor_relative_image.get_pixelv(
			cell_center + offset).r * ARMOR_CELL_EFFECT[i]
		i += 1
	armor_effect = maxf(
		armor_effect, data.armor_min_effectiveness) * data.armor_effectiveness # 1.0

	# amount    6000, 4000, 2000, 1000
	# armor     5000

	# Compute damage reduction from armor.
	var dmg_mutiplier := amount.y  / (amount.y + armor_effect * data.armor_hp_max) # 0.4444
	dmg_mutiplier = minf(dmg_mutiplier, 1.0)
	assert(is_finite(dmg_mutiplier), "minf should prevent that")
	
	# Damage that will be spread between affected cells based on distance.
	if data.armor_hp_max > 0.0:
		amount.y *= dmg_mutiplier # 1778
		var armor_cell_relative_damage := amount.y / data.armor_hp_max # 0.3555
		i = 0
		for offset in ARMOR_CELL_OFFSET:
			var point_cell := cell_center + offset # (0, 0)
			var cell := data.armor_relative_image.get_pixelv(point_cell).r
			cell -= armor_cell_relative_damage * ARMOR_CELL_EFFECT[i]
			cell = maxf(cell, 0.0) # 0.1185
			data.armor_relative_image.set_pixelv(point_cell, Color(cell, 0.0, 0.0))
			i += 1

	amount.x *= dmg_mutiplier # 2666
	data.hull_hp -= amount.x

	# TODO: Apply emp damage

	took_damage.emit(amount, cell_center)


func _integrate_angvel(wish_angvel: float, state: PhysicsDirectBodyState2D) -> void:
	var angacc_delta := data.angular_acceleration * state.step
	state.angular_velocity += clampf(
		wish_angvel - state.angular_velocity,
		-angacc_delta,
		angacc_delta)

func _integrate_linvel(wish_linvel: Vector2, state: PhysicsDirectBodyState2D) -> void:
	state.linear_velocity += (wish_linvel - state.linear_velocity).limit_length(
		data.linear_acceleration * state.step)

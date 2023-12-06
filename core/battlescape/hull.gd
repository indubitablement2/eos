extends RigidBody2D
class_name Hull


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
@export var data : HullData


## If HullSprite, will automaticaly update it.
@export var sprite : Sprite2D


@export var linear_acceleration := 800.0
@export var linear_velocity_max := 400.0
@export var angular_acceleration := 8.0
@export var angular_velocity_max := 4.0


## Changing this will change rigid body's linear/abgular velocity.
##
## Since this is used to scale many properties,
## it can not be set to 0 as a mutiplication by 0 can not be undone.
## Use a very small number like 0.01 to instead.
@export_range(0.01, 4.0, 0.01, "or_greater") var time_scale := 1.0:
	set = set_time_scale
func set_time_scale(value: float) -> void:
	var scale_properties := value / time_scale 
	time_scale = value
	
	linear_velocity *= scale_properties
	angular_velocity *= scale_properties


@export var hull_hp_max := 100.0
@export var hull_hp := 100.0

@export var armor_max := 0.0
@export var armor_effectiveness := 1.0
## Armor will keep at least this much effect even when no armor remain.
@export var armor_min_effectiveness := 0.1
## Armor cells with value [0..1] relative to armor max.
@export var armor_relative_image : Image


@export_group("Turrets modifier")
## projectile, missile, laser
## What this does is left to be interpreted by the turret.
@export var turret_range := Vector3.ONE
@export var damage_mutiplier := Vector3.ONE
@export var ammo_replenish_delay := 1.0
@export var rotation_speed := 1.0
@export var fire_delay := 1.0
## ammo, missile, laser
@export var ammo_max := Vector3.ONE
@export
var prediction_iter := 0
@export_group("")

@export_category("Movement")
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
	# Keep current linear velocity.
	NONE,
	# Keep current linear velocity.
	# Do nothing unless above max, then slow down until back to max.
	KEEP,
	# Try to reach 0 linear velocity.
	STOP,
	# Cancel our current velocity to reach position as fast as possible.
	# Does not overshoot.
	POSITION_SMOOTH,
	# Same as POSITION_SMOOTH, but always try to go at max velocity.
	POSITION_OVERSHOOT,
	# Force toward an absolute direction. -y is up.
	# Magnitude bellow 1 can be used to accelerate slower.
	# Magnitude should be clamped to 1.
	FORCE_ABSOLUTE,
	# Force toward a direction relative to current rotation. -y is forward.
	# Magnitude bellow 1 can be used to accelerate slower.
	# Magnitude should be clamped to 1.
	FORCE_RELATIVE,
}
@export var wish_linear_velocity_type := WishLinearVeloctyType.NONE
@export var wish_linear_velocity := Vector2.ZERO


## null when no target.
var target: Hull = null:
	set = set_target
func set_target(value: Hull) -> void:
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


func _init() -> void:
	armor_relative_image = data.armor_relative_image.duplicate()


#func _enter_tree() -> void:
	#detector = Battlescape.hull_area_create(self)


func _exit_tree() -> void:
	set_target(null)
	PhysicsServer2D.free_rid(detector)


func _draw() -> void:
	draw_arc(Vector2.ZERO, data.radius, 0.0, TAU, 32, Color())


func _physics_process(_delta: float) -> void:
	if Engine.is_editor_hint():
		return
	PhysicsServer2D.area_set_transform(detector, transform)


func _integrate_forces(state: PhysicsDirectBodyState2D) -> void:
	# Contacts.
	# TODO: Contact signal
	var i := 0
	while i < state.get_contact_count():
		var contact_impulse := state.get_contact_impulse(i)
		var contact_magnitude := contact_impulse.length_squared()
		if contact_magnitude > 400.0:
			var amount := sqrt(contact_magnitude)
			damage(
				Vector4(amount, amount, amount, amount),
				state.get_contact_local_position(i))
		i += 1
	
	# Angular velocity.
	var angmax := angular_velocity_max * time_scale
	match wish_angular_velocity_type:
		WishAngularVelocityType.KEEP:
			if absf(state.angular_velocity) > angmax:
				_integrate_angvel(
					clampf(state.angular_velocity, -angmax, angmax), state)
		WishAngularVelocityType.STOP:
			if (!is_zero_approx(state.angular_velocity)):
				_integrate_angvel_stop(state)
		WishAngularVelocityType.AIM_SMOOTH:
			var offset := Util.angle_up(get_angle_to(wish_angular_velocity))
			var wish_dir := signf(offset)
			var close_smooth := minf(absf(offset), 0.2) / 0.2
			close_smooth *= close_smooth * close_smooth
			
			if wish_dir == signf(state.angular_velocity):
				var time_to_target := absf(offset / state.angular_velocity)
				var time_to_stop := absf(
					state.angular_velocity / (angular_acceleration * time_scale))
				if (time_to_target < time_to_stop):
					close_smooth *= -1.0
			
			_integrate_angvel(wish_dir * angmax * close_smooth, state)
		WishAngularVelocityType.AIM_OVERSHOOT:
			var wish_dir := signf(
				Util.angle_up(get_angle_to(wish_angular_velocity)))
			_integrate_angvel(wish_dir * angmax, state)
		WishAngularVelocityType.FORCE:
			_integrate_angvel(wish_angular_velocity.x * angmax, state)
	
	# Linear velocity.
	var linmax := linear_velocity_max * time_scale
	match wish_linear_velocity_type:
		WishLinearVeloctyType.KEEP:
			if state.linear_velocity.length_squared() > linmax * linmax:
				_integrate_linvel(state.linear_velocity.limit_length(linmax), state)
		WishLinearVeloctyType.STOP:
			_integrate_linvel_stop(state)
		WishLinearVeloctyType.POSITION_SMOOTH:
			var to_position := wish_linear_velocity - position
			if to_position.length_squared() < 100.0:
				# We are on target.
				_integrate_linvel_stop(state)
			else:
				_integrate_linvel(to_position.limit_length(linmax), state)
		WishLinearVeloctyType.POSITION_OVERSHOOT:
			var to_position := wish_linear_velocity - position
			if to_position.is_zero_approx():
				to_position = Vector2(0.0, linmax)
			else:
				to_position = to_position.normalized() * linmax
			_integrate_linvel(to_position, state)
		WishLinearVeloctyType.FORCE_ABSOLUTE:
			_integrate_linvel(wish_linear_velocity * linmax, state)
		WishLinearVeloctyType.FORCE_RELATIVE:
			_integrate_linvel(
				wish_linear_velocity.rotated(rotation) * linmax, state)


func wish_angular_velocity_none() -> void:
	wish_angular_velocity_type = WishAngularVelocityType.NONE

func wish_angular_velocity_keep() -> void:
	wish_angular_velocity_type = WishAngularVelocityType.KEEP

func wish_angular_velocity_stop() -> void:
	wish_angular_velocity_type = WishAngularVelocityType.STOP

## Set angular velocity to try to face a point without overshoot.
## Point is in global space.
func wish_angular_velocity_aim_smooth(point: Vector2) -> void:
	wish_angular_velocity_type = WishAngularVelocityType.AIM_SMOOTH
	wish_angular_velocity = point

func wish_angular_velocity_aim_overshoot(point: Vector2) -> void:
	wish_angular_velocity_type = WishAngularVelocityType.AIM_OVERSHOOT
	wish_angular_velocity = point

func wish_angular_velocity_force(force: float) -> void:
	wish_angular_velocity_type = WishAngularVelocityType.FORCE
	wish_angular_velocity.x = force


func wish_linear_velocity_none() -> void:
	wish_linear_velocity_type = WishLinearVeloctyType.NONE

func wish_linear_velocity_keep() -> void:
	wish_linear_velocity_type = WishLinearVeloctyType.KEEP

func wish_linear_velocity_stop() -> void:
	wish_linear_velocity_type = WishLinearVeloctyType.STOP

func wish_linear_velocity_position_smooth(point: Vector2) -> void:
	wish_linear_velocity_type = WishLinearVeloctyType.POSITION_SMOOTH
	wish_linear_velocity = point

func wish_linear_velocity_position_overshoot(point: Vector2) -> void:
	wish_linear_velocity_type = WishLinearVeloctyType.POSITION_OVERSHOOT
	wish_linear_velocity = point

func wish_linear_velocity_force_absolute(force: Vector2) -> void:
	wish_linear_velocity_type = WishLinearVeloctyType.FORCE_ABSOLUTE
	wish_linear_velocity = force

func wish_linear_velocity_force_relative(force: Vector2) -> void:
	wish_linear_velocity_type = WishLinearVeloctyType.FORCE_RELATIVE
	wish_linear_velocity = force


## point is where damage originated in global space.
## amount:
## x: hull
## y: armor
## z: shield
## w: emp
func damage(amount: Vector4, point: Vector2) -> void:
	# Find nearest armor cell.
	var cell_center := Vector2i((to_local(point) - sprite.offset) / ARMOR_SCALE).clamp(
		ARMOR_CENTER_MIN,
		armor_relative_image.get_size() - ARMOR_CENTER_MAX)
	
	# Compute armor effectiveness from nearby cells.
	var armor_effect := 0.0
	var i := 0
	for offset in ARMOR_CELL_OFFSET:
		armor_effect += armor_relative_image.get_pixelv(
			cell_center + offset).r * ARMOR_CELL_EFFECT[i]
		i += 1
	armor_effect = maxf(armor_effect, armor_min_effectiveness) * armor_effectiveness # 1.0
	
	# amount    6000, 4000, 2000, 1000
	# armor     5000
	
	# Compute damage reduction from armor.
	var dmg_mutiplier := amount.y  / (amount.y + armor_effect * armor_max) # 0.4444
	assert(is_finite(dmg_mutiplier))
	dmg_mutiplier = minf(dmg_mutiplier, 1.0)
	
	# Damage that will be spread between affected cells based on distance.
	if armor_max > 1.0:
		amount.y *= dmg_mutiplier # 1778
		var armor_cell_relative_damage := amount.y / armor_max # 0.3555
		i = 0
		for offset in ARMOR_CELL_OFFSET:
			var point_cell := cell_center + offset # (0, 0)
			var cell := armor_relative_image.get_pixelv(point_cell).r
			cell -= armor_cell_relative_damage * ARMOR_CELL_EFFECT[i]
			cell = maxf(cell, 0.0) # 0.1185
			armor_relative_image.set_pixelv(point_cell, Color(cell, 0.0, 0.0))
			i += 1
	
	amount.x *= dmg_mutiplier # 2666
	hull_hp -= amount.x
	if sprite is HullSprite:
		sprite.took_hull_damage(amount.x, cell_center)
	
	# TODO: Apply emp damage
	
	took_damage.emit(amount, cell_center)


func _integrate_angvel(
	wish_angvel: float, state: PhysicsDirectBodyState2D) -> void:
	var angacc_delta := angular_acceleration * time_scale * state.step
	state.angular_velocity += clampf(
		wish_angvel - state.angular_velocity,
		-angacc_delta,
		angacc_delta)

func _integrate_angvel_stop(state: PhysicsDirectBodyState2D) -> void:
	var angacc_delta := angular_acceleration * time_scale * state.step
	state.angular_velocity -= clampf(
		state.angular_velocity,
		-angacc_delta,
		angacc_delta)

func _integrate_linvel(
	wish_linvel: Vector2, state: PhysicsDirectBodyState2D) -> void:
	state.linear_velocity += (wish_linvel - state.linear_velocity).limit_length(
		linear_acceleration * time_scale * state.step)

func _integrate_linvel_stop(state: PhysicsDirectBodyState2D) -> void:
	state.linear_velocity -= state.linear_velocity.limit_length(
		linear_acceleration * time_scale * state.step)


func _print_armor_cell_effect() -> void:
	var start := -ARMOR_CENTER_MIN
	var end := ARMOR_CENTER_MAX
	# [coords, distance, effect]
	var result := []
	var total := 0.0
	var max_distance := -INF
	
	for y in range(start.y, end.y):
		for x in range(start.x, end.x):
			var v := Vector2i(x, y)
			var e := Vector2(v).length()
			max_distance = maxf(max_distance, e)
			
			result.push_back([v, e])
	
	for cell in result:
		var e := maxf(max_distance + 0.2 - cell[1], 0.0)
		total += e
		cell.push_back(e)
	
	for cell in result:
		cell[2] /= total
	
	for cell in result:
		print("[Vector2i", cell[0], ", ", cell[2], "],")
	
	print(total)
	total = 0.0
	for cell in result:
		total += cell[2]
	print(total)


extends RigidBody2D
class_name Entity

## Class of everything that goes in the Battlescape.

## Ships need these nodes to work properly:
## HullArmor

signal destroyed
signal spawned_entity(entity: Entity)

@export var data: EntityData

## Changing this will scale many properties.
@export var time_scale := 1.0:
	set = set_time_scale
func set_time_scale(value: float) -> void:
	assert(value > 0.01)
	
	var scale_properties := value / time_scale 
	time_scale = value
	
	linear_velocity *= scale_properties
	angular_velocity *= scale_properties
	
	linear_acceleration *= scale_properties
	linear_velocity_max *= scale_properties
	angular_acceleration *= scale_properties
	angular_velocity_max *= scale_properties
	
	flux_dissipation_rate *= scale_properties
	
	engine_repair_rate *= scale_properties

@export var linear_acceleration := 800.0
@export var linear_velocity_max := 400.0
@export var angular_acceleration := 8.0
@export var angular_velocity_max := 4.0

@export var flux_max := 100.0
@export var flux_dissipation_rate := 10.0
var flux := 0.0

## Engine hp max is equal to entity hp max (on armor) times this.
@export var engine_hp_relative_max := 0.05
@export var engine_repair_rate := 0.05
var engine_hp := 1.0
var engine_disabled := false

# TODO
## Set by entity based on acceleration.
## Meant to be read only.
## Usually in the range 0..1, but may go higher.
var engine_flare_strength_base := 0.0
var engine_flare_strength_multiplier := 1.0

enum WishAngularVelocityType {
	## Do nothing.
	NONE,
	## Do nothing unless above max, then slow down to max.
	KEEP,
	## Try to reach 0 angular velocity.
	STOP,
	## Set angular velocity to aim at a point in global space without overshoot.
	AIM_SMOOTH,
	## Same as AIM_SMOOTH, but always try to go at max velocity.
	## Faster to integrate than AIM_SMOOTH.
	AIM_OVERSHOOT,
	## Set angular velocity to take a rotation.
	ROTATION_SMOOTH,
	## Rotate left or right. 
	## Magnitude bellow or above 1 are valid.
	FORCE,
}
@export var wish_angular_velocity_type := WishAngularVelocityType.NONE
@export var wish_angular_velocity := Vector2.ZERO
const _ANGULAR_VELOCITY_INTEGRATION_METHODS: Array[StringName] = [
	&"_angular_integrate_none",
	&"_angular_integrate_keep",
	&"_angular_integrate_stop",
	&"_angular_integrate_aim_smooth",
	&"_angular_integrate_aim_overshoot",
	&"_angular_integrate_rotation_smooth",
	&"_angular_integrate_force"]

enum WishLinearVelocityType {
	## Do nothing.
	NONE,
	## Do nothing unless above max, then slow down to max.
	KEEP,
	## Try to reach 0 linear velocity.
	STOP,
	## Cancel our current velocity to reach position as fast as possible.
	## Does not overshoot.
	POSITION_SMOOTH,
	## Same as POSITION_SMOOTH, but always try to go at max velocity.
	POSITION_OVERSHOOT,
	## Force toward an absolute direction.
	## +x is east. +y is south.
	## Magnitude bellow or above 1 are valid.
	FORCE_ABSOLUTE,
	## Force toward a direction relative to current rotation. 
	## +x is forward. +y is right.
	## Magnitude bellow or above 1 are valid.
	FORCE_RELATIVE,
}
@export var wish_linear_velocity_type := WishLinearVelocityType.NONE
@export var wish_linear_velocity := Vector2.ZERO
const _LINEAR_VELOCITY_INTEGRATION_METHODS: Array[StringName] = [
	&"_linear_integrate_none",
	&"_linear_integrate_keep",
	&"_linear_integrate_stop",
	&"_linear_integrate_position_smooth",
	&"_linear_integrate_position_overshoot",
	&"_linear_integrate_force_absolute",
	&"_linear_integrate_force_relative"]

## null when no target.
var target: Entity = null:
	set = set_target
func set_target(value: Entity) -> void:
	if target:
		target.tree_exiting.disconnect(_on_target_tree_exiting)
	target = value
	if target:
		target.tree_exiting.connect(_on_target_tree_exiting, CONNECT_ONE_SHOT)
func _on_target_tree_exiting() -> void:
	target = null

var team: BattlescapeTeam
var is_ally := false

func _init() -> void:
	can_sleep = false
	custom_integrator = true
	max_contacts_reported = 8
	contact_monitor = true
	center_of_mass_mode = RigidBody2D.CENTER_OF_MASS_MODE_CUSTOM

func _ready() -> void:
	collision_layer <<= team.team * Battlescape.COLLISION_TEAM_BIT_SIZE
	collision_mask = Battlescape.make_collision_mask(team.team, collision_mask)

func _integrate_forces(state: PhysicsDirectBodyState2D) -> void:
	
	engine_flare_strength_base = 0.0
	if engine_disabled:
		return
	
	call(_ANGULAR_VELOCITY_INTEGRATION_METHODS[wish_angular_velocity_type], state)
	call(_LINEAR_VELOCITY_INTEGRATION_METHODS[wish_linear_velocity_type], state)

func _physics_process(delta: float) -> void:
	engine_hp += delta * engine_repair_rate
	if engine_hp > 1.0:
		engine_hp = 1.0
		if engine_disabled:
			# TODO: Enable engine.
			engine_disabled = false
	elif engine_hp < 0.0:
		engine_hp = 0.0
		if !engine_disabled:
			# TODO: Disable engine.
			engine_disabled = true
	
	flux = maxf(flux - flux_dissipation_rate * delta, 0.0)
	

## If this is a ship, return its ai.
func get_ship_ai() -> ShipAI:
	return get_node("ShipAI")

func wish_angular_velocity_none() -> void:
	wish_angular_velocity_type = WishAngularVelocityType.NONE
func wish_angular_velocity_keep() -> void:
	wish_angular_velocity_type = WishAngularVelocityType.KEEP
func wish_angular_velocity_stop() -> void:
	wish_angular_velocity_type = WishAngularVelocityType.STOP
func wish_angular_velocity_aim_smooth(value: Vector2) -> void:
	wish_angular_velocity_type = WishAngularVelocityType.AIM_SMOOTH
	wish_angular_velocity = value
func wish_angular_velocity_aim_overshoot(value: Vector2) -> void:
	wish_angular_velocity_type = WishAngularVelocityType.AIM_OVERSHOOT
	wish_angular_velocity = value
func wish_angular_velocity_rotation_smooth_angle(value: float) -> void:
	wish_angular_velocity_type = WishAngularVelocityType.ROTATION_SMOOTH
	wish_angular_velocity.x = value
func wish_angular_velocity_force(value: float) -> void:
	wish_angular_velocity_type = WishAngularVelocityType.FORCE
	wish_angular_velocity.x = value

func _angular_integrate_none(_state: PhysicsDirectBodyState2D) -> void:
	pass
func _angular_integrate_keep(state: PhysicsDirectBodyState2D) -> void:
	if absf(state.angular_velocity) > angular_velocity_max:
		_integrate_angvel(clampf(state.angular_velocity, -angular_velocity_max, angular_velocity_max), state)
func _angular_integrate_stop(state: PhysicsDirectBodyState2D) -> void:
	if !is_zero_approx(state.angular_velocity):
		_integrate_angvel_stop(state)
func _angular_integrate_aim_smooth(state: PhysicsDirectBodyState2D) -> void:
	var offset := get_angle_to(wish_angular_velocity)
	var wish_dir := signf(offset)
	var close_smooth := minf(absf(offset), 0.3) / 0.3
	if wish_dir == signf(state.angular_velocity):
		var time_to_target := absf(offset / state.angular_velocity)
		var time_to_stop := absf(state.angular_velocity / angular_acceleration)
		if (time_to_target < time_to_stop):
			close_smooth = -close_smooth
	_integrate_angvel(wish_dir * angular_velocity_max * close_smooth, state)
func _angular_integrate_aim_overshoot(state: PhysicsDirectBodyState2D) -> void:
	var wish_dir := signf(get_angle_to(wish_angular_velocity))
	_integrate_angvel(wish_dir * angular_velocity_max, state)
func _angular_integrate_rotation_smooth(state: PhysicsDirectBodyState2D) -> void:
	var offset := angle_difference(rotation, wish_angular_velocity.x)
	var wish_dir := signf(offset)
	var close_smooth := minf(absf(offset), 0.3) / 0.3
	if wish_dir == signf(state.angular_velocity):
		var time_to_target := absf(offset / state.angular_velocity)
		var time_to_stop := absf(state.angular_velocity / angular_acceleration)
		if (time_to_target < time_to_stop):
			close_smooth = -close_smooth
	_integrate_angvel(wish_dir * angular_velocity_max * close_smooth, state)
func _angular_integrate_force(state: PhysicsDirectBodyState2D) -> void:
	_integrate_angvel(wish_angular_velocity.x * angular_velocity_max, state)

func _integrate_angvel(wish_angvel: float, state: PhysicsDirectBodyState2D) -> void:
	state.angular_velocity += clampf(
		wish_angvel - state.angular_velocity,
		-angular_acceleration * state.step,
		angular_acceleration * state.step)

func _integrate_angvel_stop(state: PhysicsDirectBodyState2D) -> void:
	state.angular_velocity -= clampf(
		state.angular_velocity,
		-angular_acceleration * state.step,
		angular_acceleration * state.step)

func wish_linear_velocity_none() -> void:
	wish_linear_velocity_type = WishLinearVelocityType.NONE
func wish_linear_velocity_keep() -> void:
	wish_linear_velocity_type = WishLinearVelocityType.KEEP
func wish_linear_velocity_stop() -> void:
	wish_linear_velocity_type = WishLinearVelocityType.STOP
func wish_linear_velocity_position_smooth(value: Vector2) -> void:
	wish_linear_velocity_type = WishLinearVelocityType.POSITION_SMOOTH
	wish_linear_velocity = value
func wish_linear_velocity_position_overshoot(value: Vector2) -> void:
	wish_linear_velocity_type = WishLinearVelocityType.POSITION_OVERSHOOT
	wish_linear_velocity = value
func wish_linear_velocity_force_absolute(value: Vector2) -> void:
	wish_linear_velocity_type = WishLinearVelocityType.FORCE_ABSOLUTE
	wish_linear_velocity = value
func wish_linear_velocity_force_relative(value: Vector2) -> void:
	wish_linear_velocity_type = WishLinearVelocityType.FORCE_RELATIVE
	wish_linear_velocity = value

func _linear_integrate_none(_state: PhysicsDirectBodyState2D) -> void:
	pass
func _linear_integrate_keep(state: PhysicsDirectBodyState2D) -> void:
	if state.linear_velocity.length_squared() > linear_velocity_max * linear_velocity_max:
		_integrate_linvel(state.linear_velocity.limit_length(linear_velocity_max), state)
func _linear_integrate_stop(state: PhysicsDirectBodyState2D) -> void:
	if !state.linear_velocity.is_zero_approx():
		_integrate_linvel_stop(state)
func _linear_integrate_position_smooth(state: PhysicsDirectBodyState2D) -> void:
	var to_position := wish_linear_velocity - position
	var to_position_length := to_position.length()
	if to_position_length < 30.0:
		# We are on target.
		_integrate_linvel_stop(state)
	else:
		var vel_length := state.linear_velocity.length()
		var time_to_target := to_position_length / vel_length
		var time_to_stop := vel_length / linear_acceleration
		to_position /= to_position_length
		to_position *= minf(time_to_target / time_to_stop, 1.0)
		to_position *= linear_velocity_max
		_integrate_linvel(to_position, state)
func _linear_integrate_position_overshoot(state: PhysicsDirectBodyState2D) -> void:
	var to_position := wish_linear_velocity - position
	if to_position.is_zero_approx():
		to_position = Vector2(0.0, linear_velocity_max)
	else:
		to_position = to_position.normalized() * linear_velocity_max
	_integrate_linvel(to_position, state)
func _linear_integrate_force_absolute(state: PhysicsDirectBodyState2D) -> void:
	_integrate_linvel(wish_linear_velocity * linear_velocity_max, state)
func _linear_integrate_force_relative(state: PhysicsDirectBodyState2D) -> void:
	_integrate_linvel(wish_linear_velocity.rotated(rotation) * linear_velocity_max, state)

func _integrate_linvel(wish_linvel: Vector2, state: PhysicsDirectBodyState2D) -> void:
	state.linear_velocity += (wish_linvel - state.linear_velocity).limit_length(
		linear_acceleration * state.step)

func _integrate_linvel_stop(state: PhysicsDirectBodyState2D) -> void:
	state.linear_velocity -= state.linear_velocity.limit_length(
		linear_acceleration * state.step)

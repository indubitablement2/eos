extends RigidBody2D
## A ship, fighter or missile.
class_name Hull

enum HullType {
	SHIP,
	FIGHTER,
	MISSILE,
	DEBRIS,
}
@export var hull_type := HullType.SHIP

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

## Engine hp max is equal to hull hp max (on hull armor) times this.
@export var engine_hp_relative_max := 0.05
@export var engine_repair_rate := 0.05
var engine_hp := 1.0
var engine_disabled := false

# TODO
## Set by hull based on acceleration.
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
	## Rotate left or right. 
	## Magnitude bellow or above 1 are valid.
	FORCE,
}
@export var wish_angular_velocity_type := WishAngularVelocityType.NONE
@export var wish_angular_velocity := Vector2.ZERO

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

@export var modifiers: Array[Modifiers] = []

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

## Will set collision layer and mask.
var team := 0:
	set = set_team
func set_team(value: int) -> void:
	collision_layer <<= value * 4
	collision_mask = Global.make_collision_mask(value, collision_mask)
	team = value

func _ready() -> void:
	for mod in modifiers:
		mod.apply_hull(self)

#func _input(event: InputEvent) -> void:
	#if event.is_pressed():
		#print($HullSimpleArmor.damage(100, get_global_mouse_position(), Vector4.ONE))
		#return
		#var query := PhysicsPointQueryParameters2D.new()
		#query.position = get_global_mouse_position()
		#var result := get_world_2d().direct_space_state.intersect_point(query)
		#print(result)
		#if !result.is_empty():
			#print(typeof(result[0].keys()[0]))
			#var owner_id := shape_find_owner(result[0]["shape"])
			#print(owner_id)
			#print(shape_owner_get_owner(owner_id).name)
		

func _integrate_forces(state: PhysicsDirectBodyState2D) -> void:
	
	engine_flare_strength_base = 0.0
	if engine_disabled:
		return
	
	# Angular velocity.
	match wish_angular_velocity_type:
		WishAngularVelocityType.KEEP:
			if absf(state.angular_velocity) > angular_velocity_max:
				_integrate_angvel(
					clampf(state.angular_velocity, -angular_velocity_max, angular_velocity_max), state)
		WishAngularVelocityType.STOP:
			if !is_zero_approx(state.angular_velocity):
				_integrate_angvel_stop(state)
		WishAngularVelocityType.AIM_SMOOTH:
			var offset := get_angle_to(wish_angular_velocity)
			var wish_dir := signf(offset)
			var close_smooth := minf(absf(offset), 0.2) / 0.2
			close_smooth *= close_smooth * close_smooth
			
			if wish_dir == signf(state.angular_velocity):
				var time_to_target := absf(offset / state.angular_velocity)
				var time_to_stop := absf(
					state.angular_velocity / (angular_acceleration * time_scale))
				if (time_to_target < time_to_stop):
					close_smooth *= -1.0
			
			_integrate_angvel(wish_dir * angular_velocity_max * close_smooth, state)
		WishAngularVelocityType.AIM_OVERSHOOT:
			var wish_dir := signf(get_angle_to(wish_angular_velocity))
			_integrate_angvel(wish_dir * angular_velocity_max, state)
		WishAngularVelocityType.FORCE:
			_integrate_angvel(wish_angular_velocity.x * angular_velocity_max, state)
	
	# Linear velocity.
	match wish_linear_velocity_type:
		WishLinearVelocityType.KEEP:
			if state.linear_velocity.length_squared() > linear_velocity_max * linear_velocity_max:
				_integrate_linvel(state.linear_velocity.limit_length(linear_velocity_max), state)
		WishLinearVelocityType.STOP:
			if !state.linear_velocity.is_zero_approx():
				_integrate_linvel_stop(state)
		WishLinearVelocityType.POSITION_SMOOTH:
			var to_position := wish_linear_velocity - position
			if to_position.length_squared() < 100.0:
				# We are on target.
				_integrate_linvel_stop(state)
			else:
				_integrate_linvel(to_position.limit_length(linear_velocity_max), state)
		WishLinearVelocityType.POSITION_OVERSHOOT:
			var to_position := wish_linear_velocity - position
			if to_position.is_zero_approx():
				to_position = Vector2(0.0, linear_velocity_max)
			else:
				to_position = to_position.normalized() * linear_velocity_max
			_integrate_linvel(to_position, state)
		WishLinearVelocityType.FORCE_ABSOLUTE:
			_integrate_linvel(wish_linear_velocity * linear_velocity_max, state)
		WishLinearVelocityType.FORCE_RELATIVE:
			_integrate_linvel(wish_linear_velocity.rotated(rotation) * linear_velocity_max, state)

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
	

## TODO
func destroy() -> void:
	queue_free()

## If this is a ship, return its ai.
func ship_ai() -> ShipAI:
	return get_node("ShipAI")

func wish_angular_velocity_none() -> void:
	wish_angular_velocity_type = WishAngularVelocityType.NONE
func wish_angular_velocity_keep() -> void:
	wish_angular_velocity_type = WishAngularVelocityType.KEEP
func wish_angular_velocity_stop() -> void:
	wish_angular_velocity_type = WishAngularVelocityType.STOP
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
	wish_linear_velocity_type = WishLinearVelocityType.NONE
func wish_linear_velocity_keep() -> void:
	wish_linear_velocity_type = WishLinearVelocityType.KEEP
func wish_linear_velocity_stop() -> void:
	wish_linear_velocity_type = WishLinearVelocityType.STOP
func wish_linear_velocity_position_smooth(point: Vector2) -> void:
	wish_linear_velocity_type = WishLinearVelocityType.POSITION_SMOOTH
	wish_linear_velocity = point
func wish_linear_velocity_position_overshoot(point: Vector2) -> void:
	wish_linear_velocity_type = WishLinearVelocityType.POSITION_OVERSHOOT
	wish_linear_velocity = point
func wish_linear_velocity_force_absolute(value: Vector2) -> void:
	wish_linear_velocity_type = WishLinearVelocityType.FORCE_ABSOLUTE
	wish_linear_velocity = value
func wish_linear_velocity_force_relative(value: Vector2) -> void:
	wish_linear_velocity_type = WishLinearVelocityType.FORCE_RELATIVE
	wish_linear_velocity = value

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

func _integrate_linvel(wish_linvel: Vector2, state: PhysicsDirectBodyState2D) -> void:
	state.linear_velocity += (wish_linvel - state.linear_velocity).limit_length(
		linear_acceleration * state.step)

func _integrate_linvel_stop(state: PhysicsDirectBodyState2D) -> void:
	state.linear_velocity -= state.linear_velocity.limit_length(
		linear_acceleration * state.step)

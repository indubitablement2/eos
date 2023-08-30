@tool
extends RigidBody2D
class_name Hull


const HULL_SHADER := preload("res://core/shader/hull.gdshader")


const ARMOR_CELL_EFFECT = [
	[Vector2i(-1, -1), 0.04105876015144],
	[Vector2i(0, -1), 0.12609423172741],
	[Vector2i(1, -1), 0.04105876015144],
	[Vector2i(-1, 0), 0.12609423172741],
	[Vector2i(0, 0), 0.3313880324846],
	[Vector2i(1, 0), 0.12609423172741],
	[Vector2i(-1, 1), 0.04105876015144],
	[Vector2i(0, 1), 0.12609423172741],
	[Vector2i(1, 1), 0.04105876015144],
]
const ARMOR_SCALE = 16.0
const ARMOR_SIZE_MIN = Vector2i(3, 3)
const ARMOR_CENTER_MIN = Vector2i(1, 1)
const ARMOR_CENTER_MAX = Vector2i(2, 2)


const RECENT_DAMAGE_REMOVE_RATE = 0.2
# How much hull damage / hull max to reach 1.0 recent damage.
const RECENT_DAMAGE_EFFECT = 0.1


enum Tool {
	NONE,
	VERIFY,
	CENTER_SPRITE,
	DERIVE_RADIUS_FROM_SPRITE,
}
@export
var tool := Tool.NONE:
	set = set_tool
func set_tool(value: Tool) -> void:
	if !Engine.is_editor_hint():
		return
	
	match value:
		Tool.VERIFY:
			custom_integrator = true
			max_contacts_reported = 4
			contact_monitor = true
			can_sleep = false
			
			if data == null:
				data = HullData.new()
			data._verify()
			
			linear_acceleration = data.linear_acceleration
			linear_velocity_max = data.linear_velocity_max
			angular_acceleration = data.angular_acceleration
			angular_velocity_max = data.angular_velocity_max
			time_scale = data.time_scale
			hull_hp_max = data.hull_hp_max
			armor_hp_max = data.armor_hp_max
			armor_min_effectiveness = data.armor_min_effectiveness
			
			hull_hp = hull_hp_max
			
			previous_time_scale = 1.0
			
			if data.simplified_hull:
				armor_relative_image = null
				armor_relative_texture = null
				recent_damage_image = null
				recent_damage_texture = null
				# Only remove material if it is using hull shader.
				if material != null:
					if material is ShaderMaterial:
						if material.shader == HULL_SHADER:
							material = null
			else:
				armor_relative_image = data.armor_relative_image.duplicate()
				armor_relative_image.resource_local_to_scene = true
				
				armor_relative_texture = ImageTexture.create_from_image(
					armor_relative_image)
				armor_relative_texture.resource_local_to_scene = true
				
				recent_damage_image = Image.create(
					armor_relative_image.get_width(),
					armor_relative_image.get_height(),
					false,
					Image.FORMAT_RH)
				recent_damage_image.resource_local_to_scene = true
				
				recent_damage_texture = ImageTexture.create_from_image(
					recent_damage_image)
				recent_damage_texture.resource_local_to_scene = true
				
				var mat := ShaderMaterial.new()
				mat.resource_local_to_scene = true
				mat.set_shader(HULL_SHADER)
				mat.set_shader_parameter(
					"armor_max_relative_texture", data.armor_relative_texture)
				mat.set_shader_parameter(
					"armor_relative_texture", armor_relative_texture)
				mat.set_shader_parameter(
					"recent_damage_texture", recent_damage_texture)
				material = mat
		Tool.CENTER_SPRITE:
			data.sprite_offset = data.sprite.get_size() * -0.5
		Tool.DERIVE_RADIUS_FROM_SPRITE:
			var i := data.sprite.get_size().max_axis_index()
			data.radius = data.sprite.get_size()[i] * 0.5
	
	queue_redraw()


## Should always be set.
## Shared between hulls. Do not modify at runtime.
@export
var data : HullData


@export_group("Computed")

## Used to detect when local time scale change.
@export
var previous_time_scale := 1.0

@export
var linear_acceleration := 800.0
@export
var linear_velocity_max := 400.0
@export
var angular_acceleration := 8.0
@export
var angular_velocity_max := 4.0

## Changing this will change rigid body's linear/abgular velocity.
## It will also update various time based properties like
## linear_velocity_max and angular_velocity_max.
##
## Since this is used to scale other properties,
## it can not be set to 0 as a mutiplication by 0 can not be undone.
## Use a very small number like 0.01 instead of 0.
@export_range(0.01, 4.0, 0.01, "or_greater")
var time_scale := 1.0:
	set = set_time_scale
func set_time_scale(value: float) -> void:
	time_scale = value
	
	if Engine.is_editor_hint():
		return
	
	var scale_properties := value / previous_time_scale 
	
	previous_time_scale = value
	
	linear_velocity *= scale_properties
	angular_velocity *= scale_properties

@export
var hull_hp_max := 100.0

@export
var armor_hp_max := 0.0
@export
var armor_min_effectiveness := 0.1

## null when simplified hull is set.
@export
var armor_relative_image : Image = null


@export
var hull_hp := 100.0


@export
var armor_relative_texture_dirty := false
## null when simplified hull is set.
@export
var armor_relative_texture : ImageTexture = null


# Used as an hashset of Vector2i
# Keeps track of cells that have recent damage, 
# so that we don't have to iterate over the while recent damage image.
@export
var recent_damage_set := {}
# Time since recent damage was last changed.
@export
var recent_damage_last := 0.0
## null when simplified hull is set.
@export
var recent_damage_image : Image
## null when simplified hull is set.
@export
var recent_damage_texture : ImageTexture


@export
var team := 0:
	set = set_team
func set_team(value: int) -> void:
	team = value
	
	if data.hull_class == HullData.HullClass.MISSILE:
		collision_layer = Layers.HULL_MISSILE << Layers.TEAM_OFFSET * team
	elif data.hull_class == HullData.HullClass.FIGHTER:
		collision_layer = Layers.HULL_FIGHTER << Layers.TEAM_OFFSET * team
	else:
		collision_layer = Layers.HULL_SHIP << Layers.TEAM_OFFSET * team
	
	# Invalid when not added to tree.
	if detector.is_valid():
		PhysicsServer2D.area_set_collision_layer(detector, collision_layer)
	
	team_changed.emit()
signal team_changed()


@export
var projectile_range := 1.0
var projectile_damage := 1.0
var projectile_speed := 1.0


@export_group("")

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


var detector : RID


func _enter_tree() -> void:
	if Engine.is_editor_hint():
		set_tool(Tool.VERIFY)
		return
	detector = Battlescape.hull_area_create(self)


func _exit_tree() -> void:
	if Engine.is_editor_hint():
		return
	set_target(null)
	PhysicsServer2D.free_rid(detector)


func _draw() -> void:
	draw_arc(Vector2.ZERO, data.radius, 0.0, TAU, 32, Color())
	draw_texture(data.sprite, data.sprite_offset)


func _physics_process(_delta: float) -> void:
	if Engine.is_editor_hint():
		return
	PhysicsServer2D.area_set_transform(detector, transform)


func _process(delta: float) -> void:
	if Engine.is_editor_hint():
		return
	
	if !data.simplified_hull:
		recent_damage_last += delta * time_scale
	
		# TODO: Only if we can see the sprite + offset
		if true:
			if !recent_damage_set.is_empty():
				_update_recent_damage()
				recent_damage_texture.update(recent_damage_image)
			
			if armor_relative_texture_dirty:
				armor_relative_texture_dirty = false
				armor_relative_texture.update(armor_relative_image)


func _integrate_forces(state: PhysicsDirectBodyState2D) -> void:
	# Contacts.
	# TODO: Contact signal
	var i := 0
	while i < state.get_contact_count():
		var contact_impulse := state.get_contact_impulse(i)
		var contact_magnitude := contact_impulse.length_squared()
		if contact_magnitude > 400.0:
			damage(
				sqrt(contact_magnitude),
				state.get_contact_local_position(i),
				Vector4(1.0, 1.0, 1.0, 0.25))
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


## amount and mutipliers.y should never be <= 0.
## point is where damage originated in global space.
## multipliers:
## x: hull
## y: armor
## z: shield
## w: emp
func damage(amount: float, global_point: Vector2, multipliers: Vector4) -> void:
	amount *= multipliers.y
	if amount < 0.01:
		return
	
	var remaining_dmg : float
	
	if data.simplified_hull:
		var armor := maxf(armor_hp_max, data.armor_hp_max * armor_min_effectiveness)
		var dmg_reduction := amount / (amount + armor)
		amount *= dmg_reduction
		
		var armor_dmg := amount * (armor_hp_max / data.armor_hp_max)
		remaining_dmg = amount - armor_dmg
		
		armor_hp_max -= armor_dmg
		if armor_hp_max < 0.0:
			remaining_dmg -= armor_hp_max
			armor_hp_max = 0.0
	else:
		# Find nearest armor cell.
		global_point = (to_local(global_point) - data.sprite_offset) / ARMOR_SCALE
		var local_point := Vector2i(global_point).clamp(
			ARMOR_CENTER_MIN,
			data.armor_relative_image.get_size() - ARMOR_CENTER_MAX)
		
		# Compute armor relative. [0..1]
		var armor := 0.0
		for cell in ARMOR_CELL_EFFECT:
			armor += armor_relative_image.get_pixelv(
				local_point + cell[0]).r * cell[1]
		armor = maxf(armor, armor_min_effectiveness)
		
		# Damage reduction.
		armor *= armor_hp_max
		var dmg_reduction := amount / (amount + armor)
		amount *= dmg_reduction
		
		# Apply armor damage.
		var armor_dmg := amount * (armor / armor_hp_max)
		remaining_dmg = amount - armor_dmg
		# Armor relative damage.
		armor_dmg = armor_dmg * ARMOR_CELL_EFFECT.size() / armor_hp_max
		var recent_dmg := remaining_dmg / hull_hp_max / multipliers.y
		_update_recent_damage()
		armor_relative_texture_dirty = true
		for cell in ARMOR_CELL_EFFECT:
			var point : Vector2 = local_point + cell[0]
			var v := armor_relative_image.get_pixelv(point).r
			var v2 := recent_damage_image.get_pixelv(point).r
			
			v -= armor_dmg * cell[1]
			if v < 0.0:
				var extra := v * armor_hp_max / ARMOR_CELL_EFFECT.size()
				remaining_dmg -= extra
				v2 -= extra / hull_hp_max / multipliers.y
				v = 0.0
			armor_relative_image.set_pixelv(point, Color(v, 0.0, 0.0))
			
			recent_damage_set[point] = null
			v2 += recent_dmg
			v2 *= multipliers.x
			v2 = minf(v2, 1.0)
			recent_damage_image.set_pixelv(point, Color(v2, 0.0, 0.0))
	
	remaining_dmg /= multipliers.y
	
	var hull_dmg := remaining_dmg * multipliers.x
	hull_hp -= hull_dmg
	
	var emp_dmg := remaining_dmg * multipliers.w
	# TODO: Apply emp damage
	print("emp_dmg: ", emp_dmg)


func _update_recent_damage() -> void:
	if recent_damage_last == 0.0 || recent_damage_set.is_empty():
		recent_damage_last = 0.0
		return
	
	var recent_damage_remove := recent_damage_last * RECENT_DAMAGE_REMOVE_RATE
	recent_damage_last = 0.0
	
	for key in recent_damage_set.keys():
		var new_value = recent_damage_image.get_pixelv(key).r - recent_damage_remove
		if new_value < 0.0:
			recent_damage_set.erase(key)
			recent_damage_image.set_pixelv(key, Color())
		else:
			recent_damage_image.set_pixelv(key, Color(new_value, 0.0, 0.0))


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


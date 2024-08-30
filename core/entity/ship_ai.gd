extends Area2D
class_name ShipAI

## Expects all ships to have this as a child node named ShipAI.

const AVOIDANCE_SCAN_DISTANCE := 250.0

enum ShipAIState {
	FIGHT,
	ENTRY,
	EXIT,
	FLEE,
}
var state := ShipAIState.ENTRY

@onready var entity: Entity = get_parent()

signal player_controlled_changed
var player_controlled := false:
	set = set_player_controlled
func set_player_controlled(value: bool) -> void:
	if value == player_controlled:
		return
	player_controlled = value
	if !player_controlled && _time_scale_change != 1.0:
		# Revert to local time scale.
		entity.time_scale = _time_scale_change
		Battlescape.set_time_scale(Battlescape.get_time_scale() / _time_scale_change)
		_time_scale_change = 1.0
	is_auto_pilot = true
	player_controlled_changed.emit()

var is_auto_pilot := true

var _time_scale_change := 1.0

func _ready() -> void:
	collision_mask = Battlescape.make_collision_mask(
		entity.team.team,
		collision_mask)

func _physics_process(_delta: float) -> void:
	if player_controlled && entity.time_scale != 1.0:
		# Change global time scale instead.
		_time_scale_change *= entity.time_scale
		Battlescape.set_time_scale(Battlescape.get_time_scale() / entity.time_scale)
		entity.time_scale = 1.0
	
	match state:
		ShipAIState.ENTRY:
			entity.wish_linear_velocity_force_absolute(_avoidance(Vector2.RIGHT.rotated(entity.team.entry_dir)))
			entity.wish_angular_velocity_rotation_smooth_angle(entity.team.entry_dir)
			if global_position.length() < Battlescape.node.battle_radius:
				state = ShipAIState.FIGHT
		ShipAIState.EXIT:
			entity.wish_linear_velocity_force_absolute(_avoidance(Vector2.RIGHT.rotated(-entity.team.entry_dir)))
			entity.wish_angular_velocity_rotation_smooth_angle(-entity.team.entry_dir)
		_:
			if is_auto_pilot:
				_ai()

func _ai() -> void:
	entity.wish_linear_velocity_force_absolute(_avoidance(-global_position.normalized()))
	entity.wish_angular_velocity_aim_smooth(Vector2.ZERO)

## Modify wish direction to avoid collision with nearby entities.
func _avoidance(wish_dir: Vector2) -> Vector2:
	DebugDraw.queue_draw_line(global_position, global_position + wish_dir * 200.0)
	var away_dir := Vector2.ZERO
	var away_strength := 0.0
	for other: Entity in get_overlapping_bodies():
		if other == entity:
			continue
		var from_other := global_position - other.position
		var strength := clampf(1.05 - from_other.length() / AVOIDANCE_SCAN_DISTANCE, 0.05, 1.0)
		DebugDraw.queue_draw_line(global_position, global_position + from_other.normalized() * strength * 200.0, Color.RED)
		away_dir += from_other.normalized() * strength
		away_strength += strength
	if away_strength == 0.0:
		return wish_dir
	else:
		var a:= wish_dir.slerp(away_dir.normalized(), minf(away_strength, 1.0))
		DebugDraw.queue_draw_line(global_position, global_position + a * 200.0, Color.RED)
		return a

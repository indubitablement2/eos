extends Node
class_name ShipAI

## Expects all ships to have this as a child node named ShipAI.

enum ShipAiState {
	FIGHT,
	ENTRY,
	EXIT,
	FLEE,
}
var state := ShipAiState.FIGHT

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
	auto_pilot = true
	player_controlled_changed.emit()

var auto_pilot := true

var _time_scale_change := 1.0

func _init() -> void:
	process_priority = -1

func _physics_process(_delta: float) -> void:
	if player_controlled && entity.time_scale != 1.0:
		# Change global time scale instead.
		_time_scale_change *= entity.time_scale
		Battlescape.set_time_scale(Battlescape.get_time_scale() / entity.time_scale)
		entity.time_scale = 1.0
	
	var target := Vector2.ZERO
	
	match state:
		ShipAiState.ENTRY:
			if entity.position.length_squared() < Battlescape.node.radius * Battlescape.node.radius:
				state = ShipAiState.FIGHT
			else:
				pass
		ShipAiState.EXIT:
			pass
		_:
			if auto_pilot:
				_ai()
			else:
				return
	
	

func _ai() -> void:
	pass

const AVOIDANCE_SCAN_DISTANCE := 250.0

#func _path_to_target(target: Vector2) -> void:
	#var to_target := target - entity.position
	#var target_distance := to_target.length()
	#var wish_dir := to_target / target_distance
	#
	#var away_dir := Vector2.ZERO
	#var away_strength := 0.0
	#for other: Area2D in $Sensor.get_overlapping_areas():
		#if other == $Agent:
			#continue
		#var strength = minf(1.05 - entity.position.distance_to(other.global_position) / AVOIDANCE_SCAN_DISTANCE, 1.0)
		#away_dir += (position - other.global_position).normalized() * strength
		#away_strength += strength
	#if away_dir.is_zero_approx():
		#var time_to_target := to_position.length() / vel.length()
		#var time_to_stop := vel.length() / acceleration
		#wish_dir = wish_dir * vel_max * minf(time_to_target / time_to_stop, 1.0)
	#else:
		#away_dir = away_dir.normalized()
		#wish_dir = wish_dir.slerp(away_dir, minf(away_strength, 1.0)) * vel_max
		##wish_dir = (wish_dir + away_dir).normalized() * vel_max

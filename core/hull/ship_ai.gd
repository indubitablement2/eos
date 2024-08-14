extends NavigationAgent2D
class_name ShipAI

## Expect all ships to have this as a child node named ShipAI.

@onready var hull: Hull = get_parent()

signal player_controlled_changed
var player_controlled := false:
	set = set_player_controlled
func set_player_controlled(value: bool) -> void:
	if value == player_controlled:
		return
	player_controlled = value
	if !player_controlled && _time_scale_change != 1.0:
		# Revert to local time scale.
		hull.time_scale = _time_scale_change
		Battlescape.set_time_scale(Battlescape.get_time_scale() / _time_scale_change)
		_time_scale_change = 1.0
	auto_pilot = true
	player_controlled_changed.emit()

var auto_pilot := true

var _time_scale_change := 1.0

func _init() -> void:
	process_priority = -1

func _physics_process(_delta: float) -> void:
	if player_controlled && hull.time_scale != 1.0:
		# Change global time scale instead.
		_time_scale_change *= hull.time_scale
		Battlescape.set_time_scale(Battlescape.get_time_scale() / hull.time_scale)
		hull.time_scale = 1.0
	
	if auto_pilot:
		pass


extends Node2D
class_name Battlescape

static var node: Battlescape

var num_ship_per_team: Array[int]

func _ready() -> void:
	num_ship_per_team.resize(7)
	num_ship_per_team.fill(0)
	node = self
	add_child(BattlescapePlayer.new())

func _exit_tree() -> void:
	Battlescape.set_time_scale(1.0)

static func set_time_scale(value: float) -> void:
	if is_equal_approx(value, 1.0):
		value = 1.0
	Engine.time_scale = value
	# TODO: Audio

static func set_player_time_scale(value: float) -> void:
	Engine.time_scale = value

static func get_time_scale() -> float:
	return Engine.time_scale

static func spawn_hull(
	scene: PackedScene,
	pos: Vector2,
	rot: float,
	from: Hull = null,
	target_override: Hull = null) -> Hull:
	var hull: Hull = scene.instantiate()
	hull.position = pos
	hull.rotation = rot
	
	if from:
		hull.team = from.team
		hull.modifiers = hull.modifiers
		hull.add_collision_exception_with(from)
		if target_override:
			hull.target = target_override
		else:
			hull.target = from.target
	else:
		hull.team = 0
		hull.target = target_override
	
	if hull.hull_type == Hull.HullType.SHIP:
		node.num_ship_per_team[hull.team] += 1
		hull.tree_exiting.connect(_ship_exiting.bind(hull.team))
	
	node.add_child(hull)
	return hull

static func _ship_exiting(team: int) -> void:
	node.num_ship_per_team[team] -= 1

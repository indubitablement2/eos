extends Node2D
class_name Battlescape

@export var entry_dir: Array[float] = [-PI * 0.5, PI * 0.5, 0.0, -PI]
@export var is_exit_opposite_of_entry := false
@export var is_entry_disabled := [false, false, false, false]
@export var max_ship := [30, 30, 30, 30]
@export var max_ship_point := [300, 300, 300, 300]

@export var battle_radius := 10000.0

static var node: Battlescape

var num_ship_per_team: Array[int]

static func _static_init() -> void:
	_query = PhysicsShapeQueryParameters2D.new()
	
	_arcs.resize(_ARCS_SIZE)
	_arcs[0] = RID()
	_arcs[-1] = PhysicsServer2D.circle_shape_create()
	PhysicsServer2D.shape_set_data(_arcs[-1], 1.0)
	for i in range(1, _ARCS_SIZE - 1):
		_arcs[i] = PhysicsServer2D.convex_polygon_shape_create()
		var arc := float(i) / float(_ARCS_SIZE - 1) * PI
		var num := maxi(int(roundf(float(i) / 3.0)), 4)
		if num % 2 == 0:
			num += 1
		var arr: Array[Vector2] = []
		arr.push_back(Vector2.ZERO)
		for ii in num:
			var weight := float(ii) / float(num - 1)
			var angle := lerpf(arc, -arc, weight)
			arr.push_back(Vector2.RIGHT.rotated(angle))
		PhysicsServer2D.shape_set_data(_arcs[i], PackedVector2Array(arr))

static func _static_exit() -> void:
	for rid in _arcs:
		if rid.is_valid():
			PhysicsServer2D.free_rid(rid)

func _ready() -> void:
	num_ship_per_team.resize(7)
	num_ship_per_team.fill(0)
	assert(!node)
	node = self
	add_child(BattlescapePlayer.new())

func _exit_tree() -> void:
	Battlescape.set_time_scale(1.0)

func _draw() -> void:
	for dir in entry_dir:
		const ORIGIN := Vector2(200, 200)
		var to := Vector2(100.0, 0.0).rotated(dir) + ORIGIN
		draw_line(ORIGIN, to, Color.ALICE_BLUE)
		draw_string(ThemeDB.fallback_font, to, String.num(dir, 2))

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
		hull.is_ally = from.is_ally
		hull.modifiers = hull.modifiers
		if target_override:
			hull.target = target_override
		else:
			hull.target = from.target
		for exception in from.get_collision_exceptions():
			hull.add_collision_exception_with(exception)
		hull.add_collision_exception_with(from)
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

static var _query: PhysicsShapeQueryParameters2D
const _ARCS_SIZE := 64
static var _arcs: Array[RID]
## Return an invalid RID if arc is too small.
## Return a circle if close or above PI.
## All have a radius of 1.
## Do not modify the shape. Use transform instead.
static func get_arc_shape(arc: float) -> RID:
	return _arcs[clampi(int(roundf((arc / PI) * float(_ARCS_SIZE - 1))), 0, _ARCS_SIZE - 1)]
## Same as get_arc_shape, but returns the smallest available arc instead of an invalid RID.
static func get_valid_arc_shape(arc: float) -> RID:
	return _arcs[clampi(int(roundf((arc / PI) * float(_ARCS_SIZE - 1))), 1, _ARCS_SIZE - 1)]
static func get_circle_shape() -> RID:
	return _arcs[-1]

## collider: The colliding object.
## 
## collider_id: The colliding object's ID.
## 
## rid: The intersecting object's RID.
## 
## shape: The shape index of the colliding shape.
static func intersect_circle(
	pos: Vector2,
	radius: float,
	collision_mask: int,
	exclude: Array[Hull],
	max_result := 32) -> Array[Dictionary]:
	_query.shape_rid = _arcs[-1]
	_query.transform = Transform2D(0.0, Vector2(radius, radius), 0.0, pos)
	_query.collision_mask = collision_mask
	var exclude_rid: Array[RID] = []
	exclude_rid.resize(exclude.size())
	for i in exclude.size():
		exclude_rid[i] = exclude[i].get_rid()
	_query.exclude = exclude_rid
	return node.get_world_2d().direct_space_state.intersect_shape(_query, max_result)

static func intersect_arc(
	pos: Vector2,
	radius: float,
	rot: float,
	arc: float,
	collision_mask: int,
	exclude: Array[Hull],
	max_result := 32) -> Array[Dictionary]:
	_query.shape_rid = get_valid_arc_shape(arc)
	_query.transform = Transform2D(rot, Vector2(radius, radius), 0.0, pos)
	_query.collision_mask = collision_mask
	var exclude_rid: Array[RID] = []
	exclude_rid.resize(exclude.size())
	for i in exclude.size():
		exclude_rid[i] = exclude[i].get_rid()
	_query.exclude = exclude_rid
	return node.get_world_2d().direct_space_state.intersect_shape(_query, max_result)


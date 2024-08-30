extends Node2D
class_name Battlescape

static var node: Battlescape = null

## Call finish to emit this.
signal finished

@export var battle_radius := 1000.0

var player_team: BattlescapeTeam
## Index do not correspond to team number.
## There can be multiple team 0 for example.
var teams: Array[BattlescapeTeam] = []
var time := 0.0

static func _static_entry() -> void:
	_query_shape = PhysicsShapeQueryParameters2D.new()
	_query_point = PhysicsPointQueryParameters2D.new()
	_query_ray = PhysicsRayQueryParameters2D.new()
	_query_ray.hit_from_inside = true
	
	_arcs.resize(_ARCS_SIZE)
	for i in _arcs.size():
		var arr: Array[Shape2D] = []
		_arcs[i] = arr
	_arcs[-1].push_back(CircleShape2D.new())
	_arcs[-1][0].radius = 1.0
	for i in range(1, _ARCS_SIZE - 1):
		var arc := float(i) / float(_ARCS_SIZE - 1) * PI
		var arr: Array[Vector2] = [Vector2.RIGHT]
		const STEP := 0.261799388
		var angle := 0.0
		while true:
			angle += STEP
			angle = minf(angle, arc)
			arr.push_back(Vector2.RIGHT.rotated(angle))
			arr.push_front(Vector2.RIGHT.rotated(-angle))
			if angle >= arc:
				break
		arr.push_back(Vector2.ZERO)
		if arr[0].x >= 0.0:
			_arcs[i].push_back(ConvexPolygonShape2D.new())
			_arcs[i][0].points = PackedVector2Array(arr)
		else:
			var other: Array[Vector2] = []
			while arr[0] != Vector2.RIGHT:
				other.push_back(arr.pop_front())
			other.push_back(Vector2.RIGHT)
			other.push_back(Vector2.ZERO)
			_arcs[i].push_back(ConvexPolygonShape2D.new())
			_arcs[i][0].points = PackedVector2Array(arr)
			_arcs[i].push_back(ConvexPolygonShape2D.new())
			_arcs[i][1].points = PackedVector2Array(other)

func _ready() -> void:
	assert(!node)
	node = self
	
	for child in get_children():
		if child is BattlescapeTeam:
			teams.push_back(child)
			if child.team == 0 && ! child.is_ally:
				assert(!player_team)
				player_team = child
	assert(player_team)
	
	add_child(BattlescapePlayer.new())

func _exit_tree() -> void:
	if node == self:
		node = null

func _physics_process(delta: float) -> void:
	time += delta

func _draw() -> void:
	draw_arc(Vector2.ZERO, battle_radius, 0.0 ,INF ,128 ,Color.ALICE_BLUE)


## Call for the metascape to take the result of the battlescape.
func finish() -> void:
	for team in teams:
		team.update_ship_saves()
	
	finished.emit()
	queue_free()


static func set_time_scale(value: float) -> void:
	if is_equal_approx(value, 1.0):
		value = 1.0
	Engine.time_scale = value
	# TODO: Audio

static func get_time_scale() -> float:
	return Engine.time_scale


static func spawn_from(
	entity_scene: PackedScene,
	pos: Vector2,
	rot: float,
	from: Entity,
	target_override: Entity = null) -> Entity:
	var entity: Entity = entity_scene.instantiate()
	entity.position = pos
	entity.rotation = rot
	
	entity.team = from.team
	entity.is_ally = from.is_ally
	
	if target_override:
		entity.target = target_override
	else:
		entity.target = from.target
	
	for exception in from.get_collision_exceptions():
		entity.add_collision_exception_with(exception)
	entity.add_collision_exception_with(from)
	
	node.add_child(entity)
	
	from.spawned_entity.emit(entity)
	
	return entity


const _ARCS_SIZE := 64
## Array[Array[Shape2D]
static var _arcs: Array[Array]
## Return an empty array if arc is too small.
## Return a circle if close or above PI.
## All have a radius of 1.
## Do not modify the shape. Use scale instead.
## Will return 2 Shape2D if arc is >PI/2 and isn't a full circle.
static func get_arc_shape(arc: float) -> Array[Shape2D]:
	return _arcs[clampi(int(roundf((arc / PI) * float(_ARCS_SIZE - 1))), 0, _ARCS_SIZE - 1)]
## Same as get_arc_shape, but returns the smallest available arc instead of null.
static func get_valid_arc_shape(arc: float) -> Array[Shape2D]:
	return _arcs[clampi(int(roundf((arc / PI) * float(_ARCS_SIZE - 1))), 1, _ARCS_SIZE - 1)]
static func get_circle_shape() -> Shape2D:
	return _arcs[-1][0]


static var _query_shape: PhysicsShapeQueryParameters2D
static var _query_point: PhysicsPointQueryParameters2D
static var _query_ray: PhysicsRayQueryParameters2D

static func intersect_circle(
	pos: Vector2,
	radius: float,
	collision_mask: int,
	exclude: Array[RID] = [],
	max_results := 32) -> Array[Entity]:
	_query_shape.shape = get_circle_shape()
	_query_shape.transform = Transform2D(0.0, Vector2(radius, radius), 0.0, pos)
	_query_shape.collision_mask = collision_mask
	_query_shape.exclude = exclude
	var ret: Array[Entity] = []
	for dic in node.get_world_2d().direct_space_state.intersect_shape(_query_shape, max_results):
		if !ret.has(dic["collider"]):
			ret.push_back(dic["collider"])
	return ret

static func intersect_point(
	pos: Vector2,
	collision_mask: int,
	exclude: Array[RID] = []) -> Entity:
	_query_point.position = pos
	_query_point.collision_mask = collision_mask
	_query_point.exclude = exclude
	for dic in node.get_world_2d().direct_space_state.intersect_point(_query_point, 1):
		return dic["collider"]
	return null 

## null if the ray did not intersect anything.
static var intersect_ray_entity: Entity = null
static var intersect_ray_result_position: Vector2
## The surface normal at the intersection point, or Vector2(0, 0) if the ray starts inside the shape.
static var intersect_ray_result_normal: Vector2
static func intersect_ray(
	from: Vector2,
	to: Vector2,
	collision_mask: int,
	exclude: Array[RID] = []) -> void:
	_query_ray.from = from
	_query_ray.to = to
	_query_ray.collision_mask = collision_mask
	_query_ray.exclude = exclude
	var dic := node.get_world_2d().direct_space_state.intersect_ray(_query_ray)
	if dic.is_empty():
		intersect_ray_entity = null
	else:
		intersect_ray_entity = dic["collider"]
		intersect_ray_result_position = dic["position"]
		intersect_ray_result_normal = dic["normal"]


const COLLISION_FRIEND_SHIP_S := 1 << 0
const COLLISION_FRIEND_SHIP_M := 1 << 1
const COLLISION_FRIEND_SHIP_L := 1 << 2
const COLLISION_FRIEND_SHIP_XL := 1 << 3
const COLLISION_FRIEND_SHIP := (
	COLLISION_FRIEND_SHIP_S |
	COLLISION_FRIEND_SHIP_M |
	COLLISION_FRIEND_SHIP_L |
	COLLISION_FRIEND_SHIP_XL)
const COLLISION_FRIEND_FIGHTER := 1 << 4
const COLLISION_FRIEND_MISSILE := 1 << 5
const COLLISION_FRIEND_PROJECTILE := 1 << 6
const COLLISION_FRIEND := (
	COLLISION_FRIEND_SHIP |
	COLLISION_FRIEND_FIGHTER |
	COLLISION_FRIEND_MISSILE |
	COLLISION_FRIEND_PROJECTILE)

const COLLISION_ENEMY_SHIP_S := 1 << 7
const COLLISION_ENEMY_SHIP_M := 1 << 8
const COLLISION_ENEMY_SHIP_L := 1 << 9
const COLLISION_ENEMY_SHIP_XL := 1 << 10
const COLLISION_ENEMY_SHIP := (
	COLLISION_ENEMY_SHIP_S |
	COLLISION_ENEMY_SHIP_M |
	COLLISION_ENEMY_SHIP_L |
	COLLISION_ENEMY_SHIP_XL)
const COLLISION_ENEMY_FIGHTER := 1 << 11
const COLLISION_ENEMY_MISSILE := 1 << 12
const COLLISION_ENEMY_PROJECTILE := 1 << 13
const COLLISION_ENEMY := (
	COLLISION_ENEMY_SHIP |
	COLLISION_ENEMY_FIGHTER |
	COLLISION_ENEMY_MISSILE |
	COLLISION_ENEMY_PROJECTILE)

const COLLISION_DEBRIS_S := 1 << 28
const COLLISION_DEBRIS_M := 1 << 29
const COLLISION_DEBRIS_L := 1 << 30
const COLLISION_DEBRIS_XL := 1 << 31
const COLLISION_DEBRIS := (
	COLLISION_DEBRIS_S |
	COLLISION_DEBRIS_M |
	COLLISION_DEBRIS_L |
	COLLISION_DEBRIS_XL)

const COLLISION_SHIP_S := (
	COLLISION_FRIEND_SHIP_S |
	COLLISION_FRIEND_SHIP_S << 7 |
	COLLISION_FRIEND_SHIP_S << 14 |
	COLLISION_FRIEND_SHIP_S << 21)
const COLLISION_SHIP_M := COLLISION_SHIP_S << 1
const COLLISION_SHIP_L := COLLISION_SHIP_S << 2
const COLLISION_SHIP_XL := COLLISION_SHIP_S << 3
const COLLISION_SHIP := (
	COLLISION_SHIP_S |
	COLLISION_SHIP_M |
	COLLISION_SHIP_L |
	COLLISION_SHIP_XL)
const COLLISION_FIGHTER := COLLISION_SHIP_S << 4
const COLLISION_MISSILE := COLLISION_SHIP_S << 5
const COLLISION_PROJECTILE := COLLISION_SHIP_S << 6

const COLLISION_TEAM_BIT_SIZE := 7

## Return a mask which takes into account all 4 teams and the input team.
## Input mask uses editor's friend/enemy/debris.
static func make_collision_mask(team: int, mask: int) -> int:
	team *= COLLISION_TEAM_BIT_SIZE
	
	# Enemy
	var ret := mask & COLLISION_ENEMY
	ret |= ret >> COLLISION_TEAM_BIT_SIZE
	ret |= ret << 14
	ret &= ~(COLLISION_FRIEND << team)
	# Friend
	ret |= (COLLISION_FRIEND & mask) << team
	# Debris
	ret |= mask & COLLISION_DEBRIS
	
	return ret

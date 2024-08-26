extends Node2D
class_name Battlescape

static var node: Battlescape = null

## Call finish to emit this.
signal finished

@export var battle_radius := 10000.0

var player_team: BattlescapeTeam
var teams: Array[BattlescapeTeam] = []
var time := 0.0

static func _static_entry() -> void:
	_query_shape = PhysicsShapeQueryParameters2D.new()
	_query_point = PhysicsPointQueryParameters2D.new()
	_query_ray = PhysicsRayQueryParameters2D.new()
	_query_ray.hit_from_inside = true
	
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
	Battlescape.set_time_scale(1.0)
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


static var _query_shape: PhysicsShapeQueryParameters2D
static var _query_point: PhysicsPointQueryParameters2D
static var _query_ray: PhysicsRayQueryParameters2D
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

## collider: The colliding object (Entity).
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
	exclude: Array[RID],
	max_result := 32) -> Array[Dictionary]:
	_query_shape.shape_rid = get_circle_shape()
	_query_shape.transform = Transform2D(0.0, Vector2(radius, radius), 0.0, pos)
	_query_shape.collision_mask = collision_mask
	_query_shape.exclude = exclude
	return node.get_world_2d().direct_space_state.intersect_shape(_query_shape, max_result)

static func intersect_arc(
	pos: Vector2,
	radius: float,
	rot: float,
	arc: float,
	collision_mask: int,
	exclude: Array[RID],
	max_result := 32) -> Array[Dictionary]:
	_query_shape.shape_rid = get_valid_arc_shape(arc)
	_query_shape.transform = Transform2D(rot, Vector2(radius, radius), 0.0, pos)
	_query_shape.collision_mask = collision_mask
	_query_shape.exclude = exclude
	return node.get_world_2d().direct_space_state.intersect_shape(_query_shape, max_result)

static func intersect_point(
	pos: Vector2,
	collision_mask: int,
	exclude: Array[RID],
	max_result := 32) -> Array[Dictionary]:
	_query_point.position = pos
	_query_point.collision_mask = collision_mask
	_query_point.exclude = exclude
	return node.get_world_2d().direct_space_state.intersect_point(_query_point, max_result)

## collider: The colliding object (Entity).
## 
## collider_id: The colliding object's ID.
## 
## normal: The object's surface normal at the intersection point, or Vector2(0, 0) if the ray starts inside the shape.
## 
## position: The intersection point.
## 
## rid: The intersecting object's RID.
## 
## shape: The shape index of the colliding shape.
## 
## If the ray did not intersect anything, then an empty dictionary is returned instead.
static func intersect_ray(
	from: Vector2,
	to: Vector2,
	collision_mask: int,
	exclude: Array[RID]) -> Dictionary:
	_query_ray.from = from
	_query_ray.to = to
	_query_ray.collision_mask = collision_mask
	_query_ray.exclude = exclude
	return node.get_world_2d().direct_space_state.intersect_ray(_query_ray)


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

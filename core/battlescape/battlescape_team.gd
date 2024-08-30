extends Node
class_name BattlescapeTeam

const _SPAWN_SEPARATION := 300.0

enum EntryDir {
	AUTO,
	CUSTOM,
	N,
	S,
	E,
	W,
}
@export var _entry_dir := EntryDir.AUTO
@export var entry_dir: float
## Distance outside battle_radius.
@export var entry_distance := 500.0
@export var is_exit_opposite_of_entry := false
@export var auto_entry := false
@export var max_ship := 30
@export var max_ship_point := 300

@export var ships: Array[EntitySave] = []
## Used as a hashset.
## int (ship index) : Entity (null if freed)
var spawned := {}

@export var team: int
## If this is in the same team as the previous team.
@export var is_ally := false

## Time since last ship spawn.
var last_spawn := INF
var _offset_perpendicular := 0.0
var _offset_parallel := 0.0

func _ready() -> void:
	var t := _entry_dir
	if t == EntryDir.CUSTOM:
		return
	if t == EntryDir.AUTO:
		match team:
			0:
				t = EntryDir.N
			1:
				t = EntryDir.S
			2:
				t = EntryDir.E
			3:
				t = EntryDir.W
	match t:
		EntryDir.N:
			entry_dir = -PI * 0.5
		EntryDir.S:
			entry_dir = PI * 0.5
		EntryDir.E:
			entry_dir = 0.0
		EntryDir.W:
			entry_dir = -PI

func _physics_process(delta: float) -> void:
	last_spawn += delta
	if last_spawn > 10.0:
		_offset_perpendicular = 0.0
		_offset_parallel = 0.0
	
	if auto_entry:
		for i in ships.size():
			if !spawned.has(i):
				spawn_ship(i)
				break

func spawn_ship(ship_save_idx: int) -> Entity:
	var save := ships[ship_save_idx]
	
	last_spawn = 0.0
	
	var entity: Entity = save.data.entity_scene.instantiate()
	
	spawned[ship_save_idx] = entity
	
	entity.team = self
	entity.is_ally = is_ally
	
	entity.rotation = entry_dir
	
	var wish_pos := Vector2.RIGHT.rotated(-entry_dir) * (Battlescape.node.battle_radius + entry_distance)
	var final_pos: Vector2
	while true:
		final_pos = wish_pos + wish_pos.normalized() * _offset_parallel + wish_pos.normalized().rotated(PI * 0.5) * _offset_perpendicular
		
		if _offset_perpendicular <= 0.0:
			_offset_perpendicular = absf(_offset_perpendicular) + _SPAWN_SEPARATION
			if _offset_perpendicular > _SPAWN_SEPARATION * 5.0:
				_offset_parallel += _SPAWN_SEPARATION
				_offset_perpendicular = 0.0
		else:
			_offset_perpendicular = -_offset_perpendicular
		
		if Battlescape.intersect_circle(final_pos, _SPAWN_SEPARATION, -1, [], 1).is_empty():
			break
	entity.position = final_pos
	
	# TODO: Set hull & amor
	
	for modifier in save.modifiers:
		entity.add_child(modifier.instantiate())
	
	# TODO: turrets
	
	entity.tree_exiting.connect(_entity_tree_exiting.bind(ship_save_idx))
	
	Battlescape.node.add_child(entity)
	
	return entity

func update_ship_saves() -> void:
	for ship_save_idx: int in spawned.keys():
		var entity: Entity = spawned[ship_save_idx]
		if !entity:
			continue
		_update_ship_save(entity, ships[ship_save_idx])

func _entity_tree_exiting(ship_save_idx: int) -> void:
	var save := ships[ship_save_idx]
	var entity: Entity = spawned[ship_save_idx]
	
	_update_ship_save(entity, save)
	
	spawned[ship_save_idx] = null

func _update_ship_save(entity: Entity, save: EntitySave) -> void:
	# TODO: Take hull & armor
	pass


extends Node2D
class_name BattlescapeTeam

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

var ships: Array[Entity] = []

var team: int

## Time since last ship spawn.
var last_spawn := INF
var _offset_perpendicular := 0.0
var _offset_parallel := 0.0

func _physics_process(delta: float) -> void:
	last_spawn += delta
	if last_spawn > 10.0:
		_offset_perpendicular = 0.0
		_offset_parallel = 0.0

func spawn_ship(ship_idx: int) -> Entity:
	var entity := ships[ship_idx]
	entity.process_mode = Node.PROCESS_MODE_ALWAYS
	
	last_spawn = 0.0
	entity.set_meta("spawned", true)
	
	var wish_pos := Vector2.RIGHT.rotated(-entry_dir) * (Battlescape.node.battle_radius + entry_distance)
	var final_pos: Vector2
	while true:
		final_pos = wish_pos + wish_pos.normalized() * _offset_parallel + wish_pos.normalized().rotated(PI * 0.5) * _offset_perpendicular
		
		if _offset_perpendicular <= 0.0:
			_offset_perpendicular = absf(_offset_perpendicular) + 300.0
			if _offset_perpendicular > 1500.0:
				_offset_parallel += 300.0
				_offset_perpendicular = 0.0
		else:
			_offset_perpendicular = -_offset_perpendicular
		
		if Battlescape.intersect_circle(final_pos, 300.0, -1, [], 1).is_empty():
			break
	
	# TODO: turrets
	
	# TODO: Destroyed / leaving
	#entity.tree_exiting.connect(_entity_tree_exiting.bind(entity, ship_save))
	
	return entity

#func _entity_tree_exiting(entity: Entity, ship_save: ShipSave) -> void:
	#pass

func update_entry_dir() -> void:
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

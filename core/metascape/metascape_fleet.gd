extends Node2D
class_name MetascapeFleet

signal target_reached

var is_player_owned: bool
## Call update_ships after changing this.
var ships: Array[ShipSave]
var acceleration: float
var velocity_max: float
var velocity: Vector2

## A Vector2 or a Node2D derived node.
var target
## If this slow down when close to target.
var target_smoothing := false
## Required distance from the target to reach it.
var target_reached_distance := 30.0

## ShipSave : Sprite2D
var _ships_sprite := {}

func _ready() -> void:
	update_ships()

func _physics_process(delta: float) -> void:
	var t := position
	var can_emit := false
	match typeof(target):
		TYPE_VECTOR2:
			t = target
			can_emit = true
		TYPE_OBJECT:
			t = target.position
			can_emit = true
	
	var to_position := t - position
	var to_position_length := to_position.length()
	if to_position_length < target_reached_distance:
		# We are on target.
		target = null
		velocity -= velocity.limit_length(acceleration * delta)
		if can_emit:
			target_reached.emit()
	else:
		if target_smoothing:
			var vel_length := velocity.length()
			var time_to_target := to_position_length / vel_length
			var time_to_stop := vel_length / acceleration
			to_position /= to_position_length
			to_position *= minf(time_to_target / time_to_stop, 1.0)
			to_position *= velocity_max
		else:
			to_position = to_position / to_position_length * velocity_max
		velocity += (to_position - velocity).limit_length(acceleration * delta)
	
	position += velocity * delta

func update_ships(with_explosion := false) -> void:
	var old_ships_sprite = _ships_sprite
	_ships_sprite = {}
	
	for ship in ships:
		if old_ships_sprite.has(ship):
			_ships_sprite[ship] = old_ships_sprite[ship]
			old_ships_sprite.erase(ship)
		else:
			var sp := Sprite2D.new()
			sp.texture = Global.get_entity_ship_data(ship.entity_scene.resource_path).display_sprite
			sp.position = Vector2(randf_range(-100, 100), randf_range(-100, 100))
			add_child(sp)
			_ships_sprite[ship] = sp
	
	for ship in old_ships_sprite.values():
		# TODO: Explosion
		ship.queue_free()
	
	# TODO: Derive movement stats
	velocity_max = 200.0
	acceleration = 200.0

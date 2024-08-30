extends Node2D
class_name Metascape 

static var node: Metascape

@export var radius: float

static var time: float
var event_popup_canvas_layer: CanvasLayer

var _event_popup_queue: Array[Array] = []


func _ready() -> void:
	assert(!node)
	node = self
	
	time = 1000.0
	
	event_popup_canvas_layer = CanvasLayer.new()
	event_popup_canvas_layer.name = "EventPopupCanvasLayer"
	add_child(event_popup_canvas_layer)
	
	add_child(load("res://core/metascape/metascape_pauser.tscn").instantiate())
	
	var ships: Array[EntitySave] = []
	for i in 3:
		var entity_save := EntitySave.new()
		entity_save.data = Global.entity_data.pick_random()
		ships.push_back(entity_save)
	
	var player_fleet := MetascapeFleet.new()
	player_fleet.ships = ships.duplicate()
	player_fleet.position = Vector2(randf_range(-3000, 3000), randf_range(-3000, 3000))
	player_fleet.is_player_owned = true
	add_child(player_fleet)
	
	var other_fleet := MetascapeFleet.new()
	other_fleet.ships = ships.duplicate()
	other_fleet.position = Vector2(randf_range(-3000, 3000), randf_range(-3000, 3000))
	add_child(other_fleet)
	
	Metascape.queue_event_popup("res://core/metascape/event_engage_fleet.tscn", {&"setup" : [player_fleet.name, other_fleet.name]})

func _exit_tree() -> void:
	Engine.time_scale = 1.0

func _physics_process(delta: float) -> void:
	time += delta
	
	if event_popup_canvas_layer.get_child_count() == 0 && !_event_popup_queue.is_empty():
		var arr = _event_popup_queue.pop_front()
		var event = load(arr[0]).instantiate()
		for c in arr[1].keys():
			event.callv(c, arr[1][c])
		event_popup_canvas_layer.add_child(event)

func _draw() -> void:
	draw_circle(Vector2.ZERO, radius, Color.YELLOW)


static func start_battlescape(battlescape: Battlescape) -> void:
	if Battlescape.node:
		push_error("Can't start Battlescape when one is already running")
		battlescape.queue_free()
		return
	
	Engine.time_scale = 1.0
	node.get_parent().add_child(battlescape)
	Battlescape.node.finished.connect(node._on_battlescape_finished)
	node.get_parent().remove_child(node)

func _on_battlescape_finished() -> void:
	Battlescape.node.get_parent().add_child(self)
	Battlescape.node.queue_free()
	Engine.time_scale = 1.0


## calls is a dictionary where the key is a method name (StringName)
## and its value is an array which will be used as the params for callv.
static func queue_event_popup(scene_path: String, calls := {}) -> void:
	node._event_popup_queue.push_back([scene_path, calls])


static func get_current_orbit_position(period: float, distance: float) -> Vector2:
	var t := time / period
	return Vector2(cos(t), sin(t)) * distance

static func get_orbit_position(at_time: float, period: float, distance: float) -> Vector2:
	var t := at_time / period
	return Vector2(cos(t), sin(t)) * distance

## Returns the global position the target will be at when you reach it.
static func predict_orbit_position(
	global_pos: Vector2,
	speed: float,
	target_pos: Vector2,
	target_offset: Vector2,
	period: float,
	distance: float) -> Vector2:
	for _i in 3:
		var time_to_target := global_pos.distance_to(target_pos + target_offset) / speed
		target_pos = get_orbit_position(time + time_to_target, period, distance)
	return target_pos + target_offset


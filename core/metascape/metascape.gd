extends Node2D
class_name Metascape 

static var time: float

@export var radius: float


func _ready() -> void:
	time = 1000.0

func _exit_tree() -> void:
	Metascape.set_time_scale(1.0)

func _physics_process(delta: float) -> void:
	time += delta

func _draw() -> void:
	draw_circle(Vector2.ZERO, radius, Color.YELLOW)


static func set_time_scale(value: float) -> void:
	if is_equal_approx(value, 1.0):
		value = 1.0
	Engine.time_scale = value


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

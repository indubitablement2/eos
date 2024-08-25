extends Node2D
class_name MetascapeBody

@export var distance: float
@export var period: float
@export var radius: float

func _physics_process(_delta: float) -> void:
	position = Metascape.get_current_orbit_position(period, distance)

func _draw() -> void:
	draw_circle(Vector2.ZERO, radius, Color.ALICE_BLUE)


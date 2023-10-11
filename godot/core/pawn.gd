extends Pawn
## Time units is in engine seconds.
## Many values can be scaled (except by 0) but never added or subtracted,
## as scaling can be undone in any order.
## For example, a Pawn has a base movement speed of 2.0.
## Equipping boots scales this by 1.5 (now 3.0).
## Getting sick scales this by 0.8 (now 2.4).
## Removing the boots remove 1.5 scale (now 1.6).
## Recovering from sickness removes 0.8 scale, returning to 2.0.


## In-game day lenght in real seconds.
## Default: 24 minutes.
static var DAY_LENGHT := 24.0 * 60.0
## In-game season lenght in real seconds.
## Default: 6 hours.
static var SEASON_LENGHT := 24.0 * 60.0 * 15.0
## In-game year lenght in real seconds.
## Default: 24 hours.
static var YEAR_LENGHT := 24.0 * 60.0 * 15.0 * 4.0


@export
var data : PawnData

var time := 0.0

@export
var job : Node = null


@export
var satiety_max := 50.0
@export_range(0.0, 1.0, 0.01)
var satiety_normalized := 0.75
@export
var satiety_decay_normalized := 0.00069


func _input(event: InputEvent) -> void:
	if event.is_action_pressed("primary"):
		queue_movement(Vector2i(10, 10), asd, false)


func _process(delta: float) -> void:
	time += delta
	
	satiety_normalized = maxf(
		0.0,
		satiety_normalized - satiety_decay_normalized * delta)


func asd(success: bool) -> void:
	print(success)
	if !is_moving():
		queue_movement(
			Vector2i(randi_range(4, 16), randi_range(4, 16)),
			asd,
			false)


func get_path_to_simplified() -> PackedVector2Array:
	return PackedVector2Array()



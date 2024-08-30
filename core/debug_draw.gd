extends Node2D

var _queue_draw: Array[Array] = []

func _ready() -> void:
	process_priority = 1000
	z_index = 4000

func _process(_delta: float) -> void:
	queue_redraw()

func _draw() -> void:
	for arr in _queue_draw:
		callv(arr.pop_back(), arr)
	_queue_draw.clear()

func queue_draw_line(from: Vector2, to: Vector2, color := Color.WHITE, width := -1.0, antialiased := false) -> void:
	_queue_draw.push_back([
		from,
		to,
		color,
		width,
		antialiased,
		&"draw_line"
	])

func queue_draw_string(pos: Vector2, text: String, color := Color.WHITE, alignment := HORIZONTAL_ALIGNMENT_CENTER, width: float = -1) -> void:
	_queue_draw.push_back([
		ThemeDB.fallback_font,
		pos,
		text,
		alignment,
		width,
		16,
		color,
		3,
		TextServer.DIRECTION_AUTO,
		TextServer.ORIENTATION_HORIZONTAL,
		&"draw_string"
	])

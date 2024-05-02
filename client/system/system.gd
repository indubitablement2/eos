@tool
extends Node2D
class_name System

## The server which handle this system.
@export var server := Data.SERVERS[0]
@export var links : Array[System] = []

func _ready() -> void:
	if Engine.is_editor_hint():
		set_notify_local_transform(true)
		item_rect_changed.connect(queue_redraw)

func _notification(what: int) -> void:
	if what == CanvasItem.NOTIFICATION_LOCAL_TRANSFORM_CHANGED:
		queue_redraw()

func _draw() -> void:
	var font := ThemeDB.fallback_font
	draw_string(
		font,
		Vector2(0, -10),
		name,
		HORIZONTAL_ALIGNMENT_CENTER)
	
	for other in links:
		if other.get_instance_id() > get_instance_id():
			continue
		draw_line(
			Vector2.ZERO,
			to_local(other.position),
			Color.ALICE_BLUE)


func add_link(other: System) -> void:
	remove_link(other)
	links.push_back(other)
	other.links.push_back(self)

func remove_link(other: System) -> void:
	links.erase(other)
	other.links.erase(self)
	queue_redraw()
	other.queue_redraw()

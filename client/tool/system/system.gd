@tool
extends Node2D

## The server which handle this system.
@export var server_idx : int
@export var links : Array[int]

@export var _other : int
@warning_ignore("unused_private_class_variable")
@export var _add_link : bool : set = _set_add_link
func _set_add_link(value):
	if !value:
		return
	
	_set_remove_link(true)
	
	var other = get_node("../" + str(_other))
	if !other || self == other:
		return
	
	links.push_back(_other)
	other.links.push_back(name.to_int())

@warning_ignore("unused_private_class_variable")
@export var _remove_link : bool : set = _set_remove_link
func _set_remove_link(value):
	if !value:
		return
	
	queue_redraw()
	
	links.erase(_other)
	
	var other = get_node("../" + str(_other))
	if other:
		other.links.erase(name.to_int())
		other.queue_redraw()

func _ready() -> void:
	set_notify_transform(true)

func _notification(what: int) -> void:
	if what == NOTIFICATION_TRANSFORM_CHANGED:
		queue_redraw()
		for other_idx in links:
			get_node("../" + str(other_idx)).queue_redraw()

func _draw() -> void:
	var font := ThemeDB.fallback_font
	draw_string(
		font,
		Vector2(0, -10),
		name,
		HORIZONTAL_ALIGNMENT_CENTER)
	
	for other_idx in links:
		var other = get_node("../" + str(other_idx))
		if other.get_instance_id() > get_instance_id():
			continue
		draw_line(
			Vector2.ZERO,
			to_local(other.position),
			Color.ALICE_BLUE)

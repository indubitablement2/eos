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

@warning_ignore("unused_private_class_variable")
@export var _remove_all_links : bool : set = _set_remove_all_links
func _set_remove_all_links(value):
	if !value:
		return
	
	while !links.is_empty():
		_other = links[0]
		_set_remove_link(true)
	
	_other = 0

@warning_ignore("unused_private_class_variable")
@export var _fix_all_links : bool : set = _set_fix_all_links
func _set_fix_all_links(value):
	if !value:
		return
	
	for child in get_parent().get_children():
		child.queue_redraw()
		var idx = child.name.to_int()
		var remove = []
		var seen = {}
		for other_idx in child.links:
			var other = get_node("../" + str(other_idx))
			if seen.has(other_idx):
				remove.push_back(other_idx)
				continue
			if !other:
				remove.push_back(other_idx)
				continue
			if !other.links.has(idx):
				other.links.push_back(idx)
			seen[other_idx] = null
		for other_idx in remove:
			links.erase(other_idx)

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

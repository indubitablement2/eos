extends Node2D

onready var viewport_size := get_tree().get_root().get_size()

onready var debug_info_label := $UILayer/DebugInfosLabel
onready var client := $Client
onready var camera := $FollowCamera

func _ready() -> void:
#	var vs := VisualServer
#	var item := vs.canvas_item_create()
#	vs.canvas_item_set_parent(item, get_canvas())
#	var tex = preload("res://assets/debug/PixelTextureGrid_128.png")
#	vs.canvas_item_add_texture_rect(item, Rect2(Vector2.ZERO, Vector2(100.0, 100.0)), tex.get_rid())
#
#	var item2 = vs.canvas_item_create()
#	vs.canvas_item_set_parent(item2, item)
#	vs.canvas_item_add_set_transform(item2, Transform2D.IDENTITY.translated(Vector2(-100.0, -100.0)))
#	vs.canvas_item_add_texture_rect(item2, Rect2(Vector2.ZERO, Vector2(100.0, 100.0)), tex.get_rid())
	
	print(IP.get_local_addresses())

func _process(_delta: float) -> void:
#	client.update(delta)
	
	var client_pos = Vector2.ZERO
	camera.wish_anchor = client_pos
#	camera.wish_zoom = 32.0

func _on_UpdateDebugInfosTimer_timeout() -> void:
	return
	var debug_info :String= client.get_debug_info()
	if !debug_info.empty():
		debug_info_label.set_text(debug_info)

func _on_CreateFleet_pressed() -> void:
	# Ask to create a fleet.
	# TODO: Get those from a ui.
	var starting_fleet_id = 0
	var spawn_system_id = 0
	var spawn_planet_id = 0
	client.call_deferred("starting_fleet_spawn_request", starting_fleet_id, spawn_system_id, spawn_planet_id)

func _on_CreateLocalButton_pressed() -> void:
	if client.connect_local(42):
		$UILayer/CreateLocalButton.hide()

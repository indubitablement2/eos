extends Node2D

onready var viewport_size := get_tree().get_root().get_size()

onready var def := []
onready var def_img := Image.new()
onready var def_tex := ImageTexture.new()

onready var tex_rid := SpritePacker.tex.get_rid()

onready var client := $Client

func _ready() -> void:
	$CanvasLayer/Debug/TimeDilation.connect("draw", client, "_on_draw_time_dilation", [$CanvasLayer/Debug/TimeDilation])
	$CanvasLayer/Debug/TickBuffer.connect("draw", client, "_on_draw_tick_buffer", [$CanvasLayer/Debug/TickBuffer])
	
	client.connect("ConnectionResult", self, "_on_Client_ConnectionResult")
	client.connect("HasFleetChanged", self, "_on_Client_HasFleetChanged")
	
	print(IP.get_local_addresses())

func _on_Button_pressed() -> void:
	var result = client.connect_to_server("::1", 2)
	print("Connection start result: " + str(result))
	if result:
		$CanvasLayer/Button.hide()

func _on_Client_ConnectionResult(result: bool) -> void:
	if result:
		$CanvasLayer/Button.hide()

func _on_Client_HasFleetChanged(has_fleet: bool) -> void:
	if !has_fleet:
		# TODO: Get those from a ui.
		var starting_fleet_id = 0
		var spawn_system_id = 0
		var spawn_planet_id = 0
		client.call_deferred("starting_fleet_spawn_request", starting_fleet_id, spawn_system_id, spawn_planet_id)

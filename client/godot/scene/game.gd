extends Node2D

onready var viewport_size := get_tree().get_root().get_size()

onready var debug_info_label := $UILayer/DebugInfosLabel
onready var client := $Client
onready var camera := $FollowCamera

func _ready() -> void:
	print(IP.get_local_addresses())

func _process(delta: float) -> void:
	client.update(delta)
	
	var client_pos = client.get_client_position()
	camera.wish_origin = client_pos

func _on_UpdateDebugInfosTimer_timeout() -> void:
	debug_info_label.set_text(client.get_debug_infos_string())

func _on_Client_Disconnected(reason: String) -> void:
	pass # Replace with function body.

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

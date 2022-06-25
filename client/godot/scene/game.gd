extends Node2D

# The index of the controlled fleet.
var control := -1
# The number of owned fleet.
var num_owned_fleets := 0

onready var viewport_size := get_tree().get_root().get_size()

onready var fleet_select := $CanvasLayer/FleetsSelect
onready var debug_info_label := $CanvasLayer/DebugInfosLabel

onready var client := $Client

func _ready() -> void:
	client.connect("ConnectionResult", self, "_on_Client_ConnectionResult")
#	client.connect("Disconnected", self, "_on_Client_Disconnected")
	client.connect("OwnedFleetsChanged", self, "_on_Client_OwnedFleetsChanged")
	client.connect("ControlChanged", self, "_on_Client_ControlChanged")
	
	fleet_select.connect("WishControlChanged", self, "_on_WishControlChanged")
	
	print(IP.get_local_addresses())

func _on_Button_pressed() -> void:
	var result = client.connect_to_server("::1", 2)
	print("Connection start result: " + str(result))
	if result:
		$CanvasLayer/Button.hide()

func _on_Client_ConnectionResult(result: bool) -> void:
	if result:
		$CanvasLayer/Button.hide()

func _on_Client_OwnedFleetsChanged(num: int) -> void:
	num_owned_fleets = num
	fleet_select.set_num_fleet(num)

func _on_Client_ControlChanged(index: int) -> void:
	control = index

func _on_UpdateDebugInfosTimer_timeout() -> void:
	debug_info_label.set_text(client.get_debug_infos_string())

func _on_Client_Disconnected(reason: String) -> void:
	pass # Replace with function body.

func _on_WishControlChanged(num: int):
	client.control_request(num)

func _on_CreateFleet_pressed() -> void:
	# Ask to create a fleet.
	# TODO: Get those from a ui.
	var starting_fleet_id = 0
	var spawn_system_id = 0
	var spawn_planet_id = 0
	client.call_deferred("starting_fleet_spawn_request", starting_fleet_id, spawn_system_id, spawn_planet_id)

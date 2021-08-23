extends Control


const SELF_COOP_COLOR = Color.cornflower
const COOP_COLOR = Color.cadetblue
const SELF_ATTACKER_COLOR = Color.crimson
const ATTACKER_COLOR = Color.darkred
const GHOST_COLOR = Color.dimgray
const SYSTEM_IMPORTANT_COLOR = Color.mediumseagreen
const SYSTEM_UNIMPORTANT_COLOR = Color.darkseagreen

const MAX_MESSAGE = 20
const MESSAGE_DURATION = 12.0

onready var line_edit : LineEdit = $VBoxContainer2/LineEdit
onready var label_container : VBoxContainer = $VBoxContainer2/LabelContainer
onready var chat_command : Node = $ChatCommand

var join_search_success := false


func _ready() -> void:
	var err := 0
#	err += SteamNetwork.connect("received_lobby_message", self, "_on_received_lobby_message")
#	if OS.is_debug_build():
#		err += SteamNetwork.connect("search_lobby_success", self, "_on_search_lobby_success")
#		err += SteamNetwork.connect("search_lobby_failed", self, "_on_search_lobby_failed")
	assert(err == OK)


func _unhandled_key_input(event: InputEventKey) -> void:
	if event.is_action_pressed("chat"):
		start_chatting()


#*******************************************************************************
# PUBLIC
#*******************************************************************************


func start_chatting() -> void:
	line_edit.set_visible(true)
	line_edit.grab_focus()


func stop_chatting() -> void:
	line_edit.set_visible(false)
	line_edit.release_focus()


func display_message(message:String, sender_id:int = 0) -> void:
	if label_container.get_child_count() >= MAX_MESSAGE:
		push_warning("Trying to show too many message.")
		return
	
	var label := Label.new()
	var new_massage := ""
	
	var team = SteamNetwork.lobby_members_team.get(sender_id, SteamNetwork.GHOST)
	# If I am ghost, everyone is ghost.
	if SteamNetwork.lobby_members_team.get(SteamNetwork.steam_id, SteamNetwork.GHOST) == SteamNetwork.GHOST:
		team = SteamNetwork.GHOST
	
	# System important.
	if sender_id == 0:
		label.set_modulate(SYSTEM_IMPORTANT_COLOR)
		new_massage += "[SYSTEM] "
	# Sender or self is Ghost.
	elif team == SteamNetwork.GHOST:
		label.set_modulate(GHOST_COLOR)
		new_massage += "*faint voice*: "
	# Self.
	elif sender_id == SteamNetwork.steam_id:
		if team == SteamNetwork.COOP:
			label.set_modulate(SELF_COOP_COLOR)
		else:
			label.set_modulate(SELF_ATTACKER_COLOR)
		new_massage += get_my_full_name() + ": "
	# Coop.
	elif team == SteamNetwork.COOP:
		label.set_modulate(COOP_COLOR)
		new_massage += get_friend_full_name(sender_id) + ": "
	# Attacker.
	elif team == SteamNetwork.ATTACKER:
		label.set_modulate(ATTACKER_COLOR)
		new_massage += get_friend_full_name(sender_id) + ": "
	
	new_massage += message
	label.set_text(new_massage)
	label_container.add_child(label)
	
	# Make delete timer.
	var tim := Timer.new()
	label.add_child(tim)
	if tim.connect("timeout", self, "_on_LabelTimerTimeout", [label]) != OK:
		push_error("Can not connect.")
	tim.start(MESSAGE_DURATION)


func show_and_enable(toggle:bool = true) -> void:
	set_visible(toggle)
	set_process_unhandled_key_input(toggle)
	if !toggle:
		stop_chatting()


func get_my_full_name() -> String:
	return "[" + Steam.getPersonaName() + "] " + GlobalVariable.pawn_name


func get_friend_full_name(member_id) -> String:
	return "[" + Steam.getFriendPersonaName(member_id) + "] " + "*TODO: GET OTHER PLAYER HERO NAME*"


#*******************************************************************************
# PRIVATE
#*******************************************************************************


func _send_line_edit(new_text: String) -> void:
	# TODO PREVENT MESSAGE SPAM
	if new_text.empty():
		return
	if new_text.begins_with("/"):
		_interpret_chat_command(new_text.right(1))
		return
	
	if SteamNetwork.lobby_id == 0:
		display_message(new_text, SteamNetwork.steam_id)
	else:
		SteamNetwork.send_chat_message(new_text)


func _interpret_chat_command(new_text:String) -> void:
	var args := new_text.split(" ", false, 4)
	# Pad with 0s if less argument than 3.
	while args.size() < 3:
		args.append("0")
	
	var message:String = chat_command._execute_command(args[0], args[1], args[2])
	display_message(message)


#*******************************************************************************
# SIGNAL
#*******************************************************************************


func _on_LabelTimerTimeout(label : Node) -> void:
	label.queue_free()


func _on_LineEdit_focus_entered() -> void:
	# TODO Tell other player in lobby that I am chatting.
	# They will assume I keep chatting until they receive a message, an input or after 5 seconds.
	pass


func _on_LineEdit_text_entered(new_text: String) -> void:
	_send_line_edit(new_text)
	line_edit.clear()
	stop_chatting()


func _on_search_lobby_success(found_lobbies_id:Array) -> void:
	display_message(str(found_lobbies_id))


func _on_search_lobby_failed() -> void:
	display_message("Found no lobby.")


func _on_received_lobby_message(message:String, sender_id:int) -> void:
	display_message(message, sender_id)

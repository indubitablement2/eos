extends Node


var join_on_search_success := false

onready var sc := get_script() as Script


func _ready() -> void:
	var err := 0
	if OS.is_debug_build():
		err += SteamNetwork.connect("search_lobby_success", self, "_on_search_lobby_success")
		err += SteamNetwork.connect("search_lobby_failed", self, "_on_search_lobby_failed")
	assert(err == OK)


func _execute_command(com:String, a:String, b:String) -> String:
	# Check for permission.
	if com.begins_with("C"):
		if !GlobalVariable.is_cheater:
			return "Command denied. Try /help"
	elif com.begins_with("D"):
		if !OS.is_debug_build():
			return "Command denied. Try /help"
	
	var f := funcref(self, com)
	if f.is_valid():
		return f.call_func(a, b)
	return "Invalid command. Try /help"


func _on_search_lobby_success(found_lobbies_id:Array) -> void:
	if join_on_search_success:
		join_on_search_success = false
		SteamNetwork.join_lobby(found_lobbies_id[0])


func _on_search_lobby_failed() -> void:
	join_on_search_success = false


#*******************************************************************************
# COMMAND
#*******************************************************************************


func help(_a:String, _b:String) -> String:
	var commands := []
	var Ccommands := []
	var Dcommands := []
	for i in sc.get_script_method_list():
		var n := i["name"] as String
		if !n:
			continue
		elif n.begins_with("_"):
			continue
		elif n.begins_with("C"):
			Ccommands.append(n)
		elif n.begins_with("D"):
			Dcommands.append(n)
		else:
			commands.append(n)
	
	var message := str(commands)
	if GlobalVariable.is_cheater:
		message += " " + str(Ccommands)
	if OS.is_debug_build():
		message += " " + str(Dcommands)
	
	return message


func version(_a:String, _b:String) -> String:
	var v := "Engine v."
	v += Engine.get_version_info()["string"]
	v += "\n"
	v += ProjectSettings.get_setting("application/config/name")
	v += " v." + str(GlobalVariable.GAME_VERSION)
	v += " " + GlobalVariable.GAME_VERSION_NAME
	return v


func enable_cheat(_a:String, _b:String) -> String:
	GlobalVariable.is_cheater = true
	return "WARNING: From now on any character you play on will be tagged as cheater. Restart the game to disable cheat."


#*******************************************************************************
# DEBUG COMMAND
#*******************************************************************************


func Dquit(_a:String, _b:String) -> String:
	SceneChanger.change_scene(SceneChanger.EXIT)
	return "Goodbye!"


func Dwish_team(a:String, _b:String) -> String:
	if a.is_valid_integer():
		if (int(a) >= SteamNetwork.COOP) and (int(a) <= SteamNetwork.GHOST):
			SteamNetwork.wish_team = int(a)
			return "Changed wish_team to " + a
	return "Invalid team. COOP=1, ATTACKER=2, GHOST=3"


func Dprint_lobby(_a:String, _b:String) -> String:
	var message := "Lobby ID: " + str(SteamNetwork.lobby_id)
	message += "\nMember: " + str(SteamNetwork.lobby_members.size())
	message += "/" + str(Steam.getLobbyMemberLimit(SteamNetwork.lobby_id))
	for i in SteamNetwork.lobby_members:
		message += "\nLobby member ID: " + str(i)
		message += " Name: " + Steam.getFriendPersonaName(i)
		message += " Team: " + str(SteamNetwork.lobby_members_team.get(i, 666))
	message += "\nYour wish team is: " + str(SteamNetwork.wish_team)
	return message


func Dcreate_lobby(_a:String, _b:String) -> String:
	SteamNetwork.create_lobby()
	return "Attempting to create a lobby."


func Dsearch_lobby(a:String, b:String) -> String:
	var game_wide := false
	var gw_str := "level wide"
	var result_count := 10
	if a.is_valid_integer() and bool(a):
		game_wide = bool(a)
		gw_str = "game wide"
	if b.is_valid_integer():
		result_count = int(b)
	
	if SteamNetwork.search_for_lobby(game_wide, result_count):
		return "Requested " + str(result_count) + " " + gw_str + " lobbies."
	return "Request failed."


func Djoin_lobby(a:String, b:String) -> String:
	join_on_search_success = true
	var game_wide := false
	var gw_str := "level wide"
	var result_count := 10
	if a.is_valid_integer() and bool(a):
		game_wide = bool(a)
		gw_str = "game wide"
	if b.is_valid_integer():
		result_count = int(b)
	
	if SteamNetwork.search_for_lobby(game_wide, result_count):
		return "Requested " + str(result_count) + " " + gw_str + " lobbies."
	join_on_search_success = false
	return "Request failed."

func Dleave_lobby(_a:String, _b:String) -> String:
	SteamNetwork.leave_lobby()
	return "Leaving lobby..."


func Dkick(a:String, _b:String) -> String:
	var to_kick := 0
	if a.is_valid_integer():
		to_kick = int(a)
	
	var kicked_name := Steam.getFriendPersonaName(SteamNetwork.lobby_members[to_kick])
	
	SteamNetwork.send_chat_command("kick", SteamNetwork.lobby_members[to_kick])
	
	return "Kicked " + kicked_name


#*******************************************************************************
# CHEAT COMMAND
#*******************************************************************************


func Ctemplate(_a:String, _b:String) -> String:
	
	GlobalVariable.is_cheater = true
	return ""

extends Node


var join_on_search_success := false

onready var sc := get_script() as Script


func _ready() -> void:
	var err := 0
#	if OS.is_debug_build():
#		err += SteamNetwork.connect("search_lobby_success", self, "_on_search_lobby_success")
#		err += SteamNetwork.connect("search_lobby_failed", self, "_on_search_lobby_failed")
	assert(err == OK)


func _execute_command(com:String, a:String, b:String) -> String:
	var f := funcref(self, com)
	if f.is_valid():
		return f.call_func(a, b)
	return "Invalid command. Try /help"


#func _on_search_lobby_success(found_lobbies_id:Array) -> void:
#	if join_on_search_success:
#		join_on_search_success = false
#		SteamNetwork.join_lobby(found_lobbies_id[0])
#
#
#func _on_search_lobby_failed() -> void:
#	join_on_search_success = false


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
	return v

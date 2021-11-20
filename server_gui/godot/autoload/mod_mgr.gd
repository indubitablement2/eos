extends Node


# Order in which mods are loaded. Only the first *num_enabled_mod* are relevant.
var mod_order := []
# Number of enabled mods.
var num_enabled_mod := 0

# ID of the next mod to load.
var next_loaded_mod := 0


func _ready() -> void:
	_read_config()
	validate_mod_order()


# Simply take *mod_order* and *num_enabled_mod* from config. No check other than type.
func _read_config() -> void:
	# Get *mod_order* from config.
	var mod_order_setting := ProjectSettings.get_setting(GlobalVariable.MOD_ORDER) as PoolStringArray
	if typeof(mod_order_setting) != TYPE_STRING_ARRAY:
		push_error("mod_order is not the right type.")
	else:
		mod_order = Array(mod_order_setting)
	# Get *num_enabled_mod* from config.
	var num_enabled_mod_setting := ProjectSettings.get_setting(GlobalVariable.NUM_ENABLED_MOD) as int
	if typeof(num_enabled_mod_setting) != TYPE_INT:
		push_error("num_enabled_mod is not the right type.")
	else:
		num_enabled_mod = num_enabled_mod_setting


# Validate *mod_order*.
func validate_mod_order() -> void:
	# Regroup all the mod_name.pck inside the mods folder.
	var mod_folder := []
	var dir := Directory.new()
	if dir.open(GlobalVariable.MODS_FOLDER) != OK:
		push_error("Can not access the mods folder. Aborting...")
		return
	if dir.list_dir_begin(true, true) != OK:
			push_error("Can not list dir.")
			return
	var file_name := dir.get_next()
	while file_name != "":
		if file_name.ends_with(".pck"):
			mod_folder.append(file_name.trim_suffix(".pck"))
		file_name = dir.get_next()
	
	# Check that *num_enabled_mod* is not bigger than *mod_order*.size().
	if num_enabled_mod > mod_order.size():
		num_enabled_mod = mod_order.size()
	
	# Remove missing mod.pck from *mod_order*.
	for i in mod_order.size():
		if !mod_folder.has(mod_order[i]):
			mod_order.remove(i)
			if i < num_enabled_mod:
				num_enabled_mod -= 1
	
	# Append new mod.pck to *mod_order*.
	for mod in mod_folder:
		if !mod_order.has(mod):
			mod_order.append(mod)
	
	# Save changes.
	ProjectSettings.set_setting("custom/mod/mod_order", mod_order)
	ProjectSettings.set_setting("custom/mod/num_enabled_mod", num_enabled_mod)


func set_mod_order(new_mod_order := []) -> void:
	mod_order = new_mod_order
	
	if num_enabled_mod > mod_order.size():
		num_enabled_mod = mod_order.size()
		push_warning("num_enabled_mod is bigger than mod_order.size(). Using mod_order.size().")
	
	ProjectSettings.set_setting("custom/mod/mod_order", mod_order)


func set_num_enabled_mod(new_num_enabled_mod := 0) -> void:
	num_enabled_mod = new_num_enabled_mod
	
	if num_enabled_mod > mod_order.size():
		num_enabled_mod = mod_order.size()
		push_warning("num_enabled_mod is bigger than mod_order.size(). Using mod_order.size().")
	
	ProjectSettings.set_setting("custom/mod/num_enabled_mod", num_enabled_mod)


# Import mods.
func load_mods() -> void:
	while num_enabled_mod < next_loaded_mod:
		# Load the next mod.pck.
		if ProjectSettings.load_resource_pack(GlobalVariable.MODS_FOLDER + mod_order[next_loaded_mod] + ".pck") != true:
			push_error("Could not load" + mod_order[next_loaded_mod] + ".pck.")
		next_loaded_mod += 1
	
	print_debug("Done loading mods.")


# Need to be redone using var
#func merge_json(mod_to_merge:String):
#	if !VALID_MODS.has(mod_to_merge):
#		print("ModMgr: Tried to merge json, but " + mod_to_merge + "is not valid. Ignoring...")
#		return
#
#	if config.load("user://common/config.ini") != OK:
#		return
#	var mods = config.get_value("mod", mod_to_merge, [])
#	mod_order = config.get_value("mod", "mod_order", [])
#
#	var merged
#	var file = File.new()
#	# Add core.
#	if !file.file_exists("res://core/" + mod_to_merge):
#		print("ModMgr: Tried to merge " + mod_to_merge + ", but it does not exist in core.")
#		return
#	if file.open("res://core/" + mod_to_merge, File.READ) == OK:
#		merged = JSON.parse(file.get_as_text()).get_result()
#		file.close()
#
#	if mods.empty():
#		return merged
#	var mod_order_thin = mod_order.duplicate()
#	for i in mod_order_thin:
#		if !mods.has(i):
#			mod_order_thin.erase(i)
#
#	# Add mods.
#	for mod_name in mod_order_thin:
#		if file.open("user://mods/" + mod_name + "/" + mod_to_merge, File.READ) != OK:
#			file.close()
#			print("ModMgr: Could not open user://mods/" + mod_name + "/" + mod_to_merge)
#			continue
#
#		var content = JSON.parse(file.get_as_text()).get_result()
#		file.close()
#		if !content:
#			continue
#		for i in content.keys():
#			merged[i] = content[i]
#	return merged

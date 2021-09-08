extends Node
# Setup the user:// directory when they do not exist.
# Run once then free itself.


func _init() -> void:
	var dir := Directory.new()
	
	var base_name := "/Chaos Cascade v." + str(GlobalVariable.GAME_VERSION).pad_zeros(4)
	
	# Base folders.
	if !dir.dir_exists(GlobalVariable.SAVES_FOLDER):
		if dir.make_dir_recursive(GlobalVariable.SAVES_FOLDER) != OK:
			push_error("Error trying to make " + GlobalVariable.SAVES_FOLDER)
	if !dir.dir_exists(GlobalVariable.MODS_FOLDER + base_name):
		if dir.make_dir_recursive(GlobalVariable.MODS_FOLDER + base_name) != OK:
			push_error("Error trying to make " + GlobalVariable.MODS_FOLDER)
	
	# Copy base to user://mods folder.
	if dir.open(GlobalVariable.MODS_FOLDER) != OK:
		push_error("Can not open ")
		return
	dir.list_dir_begin()
	
	queue_free()

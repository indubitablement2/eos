extends Node
# Final cleanup before quitting.


func _ready() -> void:
	# Save config.
	if !OS.is_debug_build():
		if ProjectSettings.save_custom("res://override.cfg") != OK:
			push_error("Could not save override.cfg.")
	
	# Quit
	print_debug("Exit done.")
	get_tree().quit()

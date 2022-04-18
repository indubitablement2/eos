extends Control
# Entry point for the programme.


onready var progress_label := $ProgressBar/Label
onready var progress_bar := $ProgressBar


func _ready() -> void:
	progress_bar.set_value(0)
	progress_label.set_text("Loading mods")
	ModMgr.load_mods()
	
	# Done. Go to main menu.
	SceneChanger.change_scene(SceneChanger.MAIN_MENU)
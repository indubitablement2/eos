extends Control


# FSM
enum States {MAIN_MENU, OPTION, NEW_GAME, EXIT, START}
var current_state := 0
var previous_state := 0

func _unhandled_input(event: InputEvent) -> void:
func _ready() -> void:
	for child in get_children():
		child.set_visible(false)
	$MainMenu.set_visible(true)


#*******************************************************************************
# FSM
#*******************************************************************************


func _exit_state() -> void:
	match previous_state:
		States.MAIN_MENU:
			$MainMenu.set_visible(false)
		States.OPTION:
			$OptionMenu.set_visible(false)


func _enter_state() -> void:
	match current_state:
		States.MAIN_MENU:
			$MainMenu.set_visible(true)
		States.NEW_GAME:
			pass
		States.START:
			SceneChanger.change_scene(SceneChanger.GAME)
		States.EXIT:
			SceneChanger.change_scene(SceneChanger.EXIT)
		States.OPTION:
			$OptionMenu.set_visible(true)


func _set_state(transition:int) -> void:
	if transition < States.size() and transition >= 0:
		previous_state = current_state
		current_state = transition
		_exit_state()
		_enter_state()
	else:
		push_warning("Transition state is not valid.")


#*******************************************************************************
# SIGNAL
#*******************************************************************************


func _on_New_pressed() -> void:
	pass # Replace with function body.


func _on_Exit_pressed() -> void:
	_set_state(States.EXIT)


func _on_StartDebug_pressed() -> void:
	_set_state(States.START)


func _on_Option_pressed() -> void:
	_set_state(States.OPTION)

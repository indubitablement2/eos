extends Node2D

var radius_min := 32.0
var radius_max := 64.0
var num_try := 1
var brush_size := 512.0 
var wait := 4
var min_distance := 20.0

var hold_start_point := Vector2.ZERO

var current_load_state = load_state.NONE
enum load_state {
	NONE,
	FULL,
	SYSTEMS
	}

var current_click_mode = click_mode.SELECT
enum click_mode {
	SELECT,
	GENERATE,
}

onready var editor := $Editor

func _ready() -> void:
	editor.set_camera($Camera2D)

func _unhandled_input(event: InputEvent) -> void:
	if event.is_action_pressed("primary"):
		wait = 0;
		if current_click_mode == click_mode.SELECT:
			editor.select()
			hold_start_point = get_global_mouse_position()
	elif event.is_action_released("primary"):
		editor.toggle_moving_selected(false)
	elif event.is_action_pressed("secondary"):
		editor.select()
		editor.delete_selected()
	
	if Input.is_action_pressed("primary"):
		match current_click_mode:
			click_mode.SELECT:
				if hold_start_point.distance_squared_to(get_global_mouse_position()) > 1.0 * $Camera2D.zoom.x:
					editor.toggle_moving_selected(true)
			click_mode.GENERATE:
				if wait == 0:
					editor.generate(radius_min, radius_max, num_try, brush_size, min_distance)
				wait = (wait + 1) % 20

func _draw() -> void:
	if current_click_mode == click_mode.GENERATE:
		draw_arc(get_global_mouse_position(), brush_size, 0.0, TAU, 32, Color.aliceblue)

func _process(_delta: float) -> void:
	$CanvasLayer/Control/HBoxContainer/Tick.set_text("tick: " + str(editor.get_tick())) 
	$CanvasLayer/Control/HBoxContainer/NumSystem.set_text(str(editor.get_num_system()) + " systems")
	$CanvasLayer/Control/HBoxContainer/Bound.set_text("Systems bound: " + str(editor.get_bound()))
	update()

func _on_RadiusMin_text_changed(new_text: String) -> void:
	radius_min = clamp(float(new_text), 1.0, radius_max - 0.1)

func _on_RadiusMax_text_changed(new_text: String) -> void:
	radius_max = max(float(new_text), radius_min + 0.1)

func _on_Try_text_changed(new_text: String) -> void:
	num_try = int(max(float(new_text), 1.0))

func _on_Size_text_changed(new_text: String) -> void:
	brush_size = max(float(new_text), 0.1)

func _on_TimeMultiplier_text_changed(new_text: String) -> void:
	editor.set_time_multiplier(max(float(new_text), 0.0))

func _on_Export_pressed() -> void:
	$CanvasLayer/Control/Save.popup()

func _on_Save_confirmed() -> void:
	var data : PoolByteArray = editor.export_data()
	if data.empty():
		push_warning("can not save. no data")
		return
	
	var path = $CanvasLayer/Control/Save.get_current_path()
	var file = File.new()
	if file.open(path + ".bin", File.WRITE) != OK:
		push_error("error can not open file")
		file.close()
		return
	file.store_buffer(data)
	file.close()
	print("saved data to " + path)

func _on_Hide_pressed() -> void:
	$CanvasLayer/Control/VBoxContainer.set_visible(!$CanvasLayer/Control/VBoxContainer.visible)

func _on_Import_pressed() -> void:
	current_load_state = load_state.FULL
	$CanvasLayer/Control/Load.popup()

func _on_ImportSystems_pressed() -> void:
	current_load_state = load_state.SYSTEMS
	$CanvasLayer/Control/Load.popup()

func _on_Load_confirmed() -> void:
	var file = File.new()
	var path = $CanvasLayer/Control/Load.get_current_path()
	if file.open(path, File.READ) != OK:
		file.close()
		current_load_state = load_state.NONE
		push_error("can not open file" + path)
		return
	
	var data = file.get_buffer(file.get_len())
	
	var result := false
	match current_load_state:
		load_state.FULL:
			result = editor.load_data(data)
		load_state.SYSTEMS:
			result = editor.load_systems(data)
	file.close()
	current_load_state = load_state.NONE
	if !result:
		push_warning("can not load")
	else:
		print("loaded successfuly")

func _on_SetTick_text_changed(new_text: String) -> void:
	var v = int(new_text)
	editor.set_tick(v)

func _on_Select_pressed() -> void:
	set_click_mode(click_mode.SELECT)

func _on_Generate_pressed() -> void:
	set_click_mode(click_mode.GENERATE)

func set_click_mode(m) -> void:
	current_click_mode = m
	
	match current_click_mode:
		click_mode.SELECT:
				$CanvasLayer/Control/VBoxContainer/HBoxContainer2/Generate.set_pressed(false)
		click_mode.GENERATE:
				$CanvasLayer/Control/VBoxContainer/HBoxContainer2/Select.set_pressed(false)

func _on_MinDistance_text_changed(new_text: String) -> void:
	min_distance = max(float(new_text), 0.0 )

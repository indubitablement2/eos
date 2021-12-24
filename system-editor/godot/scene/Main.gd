extends Node2D

var in_seed := -1
var bound := 1024.0
var radius_min := 32.0
var radius_max := 128.0
var min_distance := 16.0
var density := 1.0
var size := 1.0 

var hold_start_point := Vector2.ZERO

var current_save_state = save_state.NONE
enum save_state {
	NONE,
	MERGE,
	EDITOR_SAVE,
	FIRST_SAVE
	}

var current_load_state = load_state.NONE
enum load_state {
	NONE,
	EDITOR,
	FIRST
	}

func _ready() -> void:
	$SystemEditor.set_camera($Camera2D)

func _unhandled_input(event: InputEvent) -> void:
	if event.is_action_pressed("primary"):
		$SystemEditor.select()
		hold_start_point = get_global_mouse_position()
	elif event.is_action_released("primary"):
		$SystemEditor.toggle_moving_selected(false)
	elif event.is_action_pressed("secondary"):
		$SystemEditor.select()
		$SystemEditor.delete_selected()
	
	if Input.is_action_pressed("primary"):
		if hold_start_point.distance_squared_to(get_global_mouse_position()) > 1.0 * $Camera2D.zoom.x:
			$SystemEditor.toggle_moving_selected(true)

func _process(_delta: float) -> void:
#	if Input.is_action_pressed("primary"):
#		if hold_start_point.distance_squared_to(get_global_mouse_position()) > 1.0 * $Camera2D.zoom.x:
#			$SystemEditor.toggle_moving_selected(true)
	$CanvasLayer/Control/HBoxContainer/Tick.set_text("tick: " + str($SystemEditor.get_tick())) 

func _draw() -> void:
	var part := TAU / 20.0
	for i in 10:
		var start := part * float(i) * 2.0
		draw_arc(Vector2.ZERO, bound, start, start + part, 3, Color.aliceblue, 0.5)

func set_bound(b:float) -> void:
	bound = b
	update()

func _on_Generate_pressed() -> void:
	$SystemEditor.generate(in_seed, bound, radius_min, radius_max, min_distance, density, size)
	$SystemEditor.update()

func _on_bound_text_changed(new_text: String) -> void:
	var b := float(new_text)
	b = max(b, 1.0)
	set_bound(b)

func _on_Seed_text_changed(new_text: String) -> void:
	in_seed = int(new_text)

func _on_RadiusMin_text_changed(new_text: String) -> void:
	radius_min = clamp(float(new_text), 1.0, radius_max - 0.1)

func _on_RadiusMax_text_changed(new_text: String) -> void:
	radius_max = max(float(new_text), radius_min + 0.1)

func _on_DistanceMin_text_changed(new_text: String) -> void:
	min_distance = max(float(new_text), 0.0)

func _on_Density_text_changed(new_text: String) -> void:
	density = clamp(float(new_text), 0.0, 5.0)

func _on_Size_text_changed(new_text: String) -> void:
	size = max(float(new_text), 0.1)

func _on_TimeMultiplier_text_changed(new_text: String) -> void:
	$SystemEditor.set_time_multiplier(max(float(new_text), 0.0))

func _on_Merge_pressed() -> void:
	current_save_state = save_state.MERGE
	$CanvasLayer/Control/Save.popup()

func _on_SaveEditor_pressed() -> void:
	current_save_state = save_state.EDITOR_SAVE
	$CanvasLayer/Control/Save.popup()

func _on_SaveFinal_pressed() -> void:
	current_save_state = save_state.FIRST_SAVE
	$CanvasLayer/Control/Save.popup()

func _on_Save_confirmed() -> void:
	var data : PoolByteArray
	match current_save_state:
		save_state.EDITOR_SAVE:
			data = $SystemEditor.export_editor_systems()
		save_state.FIRST_SAVE:
			data = $SystemEditor.export_first_systems()
		save_state.MERGE:
			data = $SystemEditor.merge()
	current_save_state = save_state.NONE
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

func _on_LoadEditor_pressed() -> void:
	current_load_state = load_state.EDITOR
	$CanvasLayer/Control/Load.popup()

func _on_LoadOriginal_pressed() -> void:
	current_load_state = load_state.FIRST
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

	var result = false
	match current_load_state:
		load_state.EDITOR:
			result = $SystemEditor.load_editor_systems(data)
		load_state.FIRST:
			result = $SystemEditor.load_first_systems(data)
	file.close()
	current_load_state = load_state.NONE
	if !result:
		push_warning("can not load")
	else:
		print("loaded successfuly")

	current_save_state = save_state.NONE
	print("saved data to " + path)

func _on_SetTick_text_changed(new_text: String) -> void:
	var v = int(new_text)
	$SystemEditor.set_tick(v)

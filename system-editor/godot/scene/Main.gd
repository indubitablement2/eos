extends Node2D

var in_seed := -1
var bound := 1024.0
var radius_min := 32.0
var radius_max := 128.0
var min_distance := 16.0
var density := 1.0
var size := 1.0 


func _process(delta: float) -> void:
	$CanvasLayer/Control/HBoxContainer/Tick.set_text("tick: " + str($SystemEditor.get_tick())) 
	var r = get_tree().get_root().get_visible_rect()
	r.position += $Camera2D.position
	r.size *= $Camera2D.zoom
	r.position -= r.size * 0.5
	$SystemEditor.set_viewport_rect(r)

func _draw() -> void:
	draw_circle(Vector2.ZERO, bound, Color(1.0, 1.0, 1.0, 0.05))


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

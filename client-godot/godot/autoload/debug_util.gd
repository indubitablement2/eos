extends Node
# Some helper function for debugging.

const MAX_DRAW = 100
var num_draw := 0

var active_label := []


func _process(_delta: float) -> void:
	var to_remove := []
	for i in active_label.size():
		if is_instance_valid(active_label[i][0]) and is_instance_valid(active_label[i][1]):
			active_label[i][1].set_text(str(active_label[i][0].get(active_label[i][2])))
		else:
			to_remove.append(i)
	
	for i in to_remove:
		active_label.remove(i)


func hook_label(target : Node, property : String, offset : Vector2 = Vector2.ZERO, duration : float = 100000) -> void:
	var found := false
	for i in target.get_property_list():
		if property == i["name"]:
			found = true
			break
	
	if !found:
		print_debug("Could not find " + property + " to hook label.")
		return
	
	var label := Label.new()
	target.add_child(label)
	label.set_position(offset)
	label.set_align(Label.ALIGN_CENTER)
	label.set_h_grow_direction(Control.GROW_DIRECTION_BOTH)
	label.set_anchors_preset(Control.PRESET_CENTER)
	
	active_label.append([target, label, property])
	
	var tim := Timer.new()
	add_child(tim)
	if tim.connect("timeout", self, "_on_LabelTimerTimeout", [tim, label]) != OK:
		push_error("Can not connect.")
	
	tim.start(duration)


func draw_vector(at : Vector2, draw : Vector2, duration : float = 1.0) -> void:
	var sp := _can_draw(duration)
	if !sp:
		return
	
	sp.set_texture(preload("res://texture/debug/arrow_unit.png"))
	sp.set_position(at)
	sp.set_rotation(draw.angle() + PI * 0.5)
	
	if is_equal_approx(draw.length(), 1.0):
		sp.set_modulate(Color.green)
	else:
		sp.set_modulate(Color.red)
		sp.scale.y = draw.length() * 0.25


func draw_point(at : Vector2, scale : float = 1.0, duration : float = 1.0) -> void:
	var sp := _can_draw(duration)
	if !sp:
		return
	
	sp.set_texture(preload("res://texture/debug/target.png"))
	sp.set_position(at)
	sp.set_scale(Vector2(scale, scale))


func draw_line(from : Vector2, to : Vector2, duration : float = 1.0, col : Color = Color.white) -> void:
	var sp := _can_draw(duration)
	if !sp:
		return
	
	sp.set_texture(preload("res://texture/debug/pixel.png"))
	sp.scale.x = (to - from).length()
	sp.set_position(from.move_toward(to, (to - from).length() * 0.5))
	sp.set_rotation(to.angle_to_point(from))
	sp.set_modulate(col)


func _can_draw(duration : float = 1.0) -> Sprite:
	if num_draw >= MAX_DRAW:
		return null
	num_draw += 1
	
	var tim := Timer.new()
	var sp := Sprite.new()
	
	add_child(tim)
	add_child(sp)
	
	if tim.connect("timeout", self, "_on_DrawTimerTimeout", [tim, sp]) != OK:
		push_error("Can not connect.")
	
	sp.set_z_index(256)
	tim.start(duration)
	return sp



func _on_DrawTimerTimeout(tim : Node, sp : Node) -> void:
	num_draw -= 1
	tim.queue_free()
	sp.queue_free()


func _on_LabelTimerTimeout(tim : Node, label : Node) -> void:
	tim.queue_free()
	label.queue_free()

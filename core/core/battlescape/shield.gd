extends Area2D
class_name Shield


var hull : Hull


var arc_max := PI
var open_rate := 1.0

var rotation_speed := 4.0
var can_rotate := false


var arc := 0.0
var close_amount := 1.0


## Rotation in local space.
var wish_angle := 0.0


var colliding : Array[Hull] = []


@onready
var circle : CircleShape2D = get_child(0).shape


func _enter_tree() -> void:
	# Should always be a child of a Hull.
	hull = get_parent()
	_on_hull_team_changed()
	hull.team_changed.connect(_on_hull_team_changed)


func _draw() -> void:
	if arc > 0.01:
		draw_arc(Vector2.ZERO,
			circle.radius,
			-arc,
			arc,
			int(0.12 * arc * circle.radius) + 2,
			Color(0.0, 0.0, 1.0, 1.0 - close_amount),
			4.0,
			true)


func _process(delta: float) -> void:
	var scaled_delta := hull.time_scale * delta
	
	# Open/close
	if close_amount > 0.0:
		close_amount += scaled_delta
		queue_redraw()
		
		if close_amount > 1.0:
			arc = 0.0
			close_amount = 1.0
			monitoring = false
			set_process_mode(Node.PROCESS_MODE_DISABLED)
	elif arc < arc_max:
		arc += open_rate * scaled_delta
		queue_redraw()
		
		if arc > arc_max:
			arc = arc_max
	
	# Rotate
	if can_rotate && arc < PI:
		var wish_angle_change := wish_angle - rotation
		if wish_angle_change > PI:
			wish_angle_change -= TAU
		elif wish_angle_change < -PI:
			wish_angle_change += TAU
		var rotation_speed_delta := rotation_speed * scaled_delta
		rotate(clampf(wish_angle_change, -rotation_speed_delta, rotation_speed_delta))
	
	# Collision
	


func is_open() -> bool:
	return close_amount == 0.0


func toggle(open: bool) -> void:
	if open:
		if close_amount == 1.0:
			if can_rotate:
				rotation = wish_angle
			close_amount = 0.0
			monitoring = true
			set_process_mode(Node.PROCESS_MODE_INHERIT)
	else:
		if close_amount == 0.0:
			close_amount = 0.01


func is_point_in_arc(point: Vector2) -> bool:
	if close_amount == 1.0:
		return false
	elif arc < PI:
		return absf(get_angle_to(point)) < arc
	else:
		return true


func _on_hull_team_changed() -> void:
	var team_offset := hull.team * Layers.TEAM_OFFSET
	if hull.data.hull_class >= HullData.HullClass.FRIGATE:
		collision_layer = Layers.DETECTOR_LARGE << team_offset
	else:
		collision_layer = Layers.DETECTOR_SMALL << team_offset


func _on_body_entered(body: Hull) -> void:
	if body == hull:
		return
	colliding.push_back(body)


func _on_body_exited(body: Hull) -> void:
	colliding.erase(body)


# TODO: notify bullet
func _on_area_entered(area: Area2D) -> void:
	pass # Replace with function body.

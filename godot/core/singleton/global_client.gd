#extends Client
#
##func _notification(what: int) -> void:
##	if what == NOTIFICATION_WM_CLOSE_REQUEST:
##		clear_data()
#
#enum ENTITY_TYPE {
#	Ship
#}
#
#func load_data() -> void:
#	clear_data()
#
#	# [[path, node]]
#	var nodes := []
#	var dirs :Array[String] = ["res:/"]
#	while !dirs.is_empty():
#		var current_path :String = dirs.pop_back() + "/"
#		var access := DirAccess.open(current_path)
#		access.list_dir_begin()
#		var file_name := access.get_next()
#		while file_name:
#			var file_path := current_path + file_name
#			if access.current_is_dir():
#				dirs.push_back(file_path)
#			else:
#				var node := _try_load_path(file_path)
#				if node:
#					nodes.push_back([file_path, node])
#			file_name = access.get_next()
#
#	nodes.sort_custom(_sort_nodes)
#
#	for e in nodes:
#		_build_entity_data(e[0], e[1])
#
#func _try_load_path(path: String) -> EntityData:
#	var ext := path.get_extension()
#	if ext != "tscn" && ext != "scn":
#		return null
#
#	var res := load(path) as PackedScene
#	if !res:
#		return null
#	if !res.can_instantiate():
#		return null
#
#	var node := res.instantiate()
#	if node is EntityData:
#		return node
#	else:
#		node.free()
#		return null
#
#func _sort_nodes(a, b) -> bool:
#	return a[0] < b[0]
#
#func _build_entity_data(path: String, e: EntityData) -> void:
#	var b := EntityDataBuilder.new()
#
#	b.set_path(path)
#
#	b.set_linear_acceleration(e.linear_acceleration)
#	b.set_angular_acceleration(e.angular_acceleration)
#	b.set_max_linear_velocity(e.max_linear_velocity)
#	b.set_max_angular_velocity(e.max_angular_velocity)
#
#	b.set_simulation_script(e.simulation_script)
#
#	b.set_hull(e.hull)
#	b.set_armor(e.armor)
#
#	b.set_aproximate_radius(e.aproximate_radius)
#
#	# Make sure there is a sprite
#	if !e.sprite:
#		var sp := Sprite2D.new()
#		e.add_child(sp)
#		e.sprite = sp
#
#	# Make sure the srpite has a texture
#	if !e.sprite.texture:
#		e.sprite.set_texture(preload("res://textures/error.png"))
#
#	var child_sprite_idx := 0
#	while true:
#		if e.get_child(child_sprite_idx) == e.sprite:
#			break
#		child_sprite_idx += 1
#	b.set_child_sprite_idx(child_sprite_idx)
#
#	# Handle if this is a ship
#	if e.ship_data:
#		b.set_ship_display_name(e.ship_data.display_name)
#		b.set_ship_texture(e.sprite.texture)
#
#	# Set the shape
#	if e.collision_shape is CollisionShape2D:
#		var c :CollisionShape2D = e.collision_shape
#		if c.shape is CircleShape2D:
#			var s : CircleShape2D = c.shape
#			b.set_shape_circle(
#				s.radius,
#				e.density,
#				e.entity_type,
#			)
#		elif c.shape is RectangleShape2D:
#			var s : RectangleShape2D = c.shape
#			b.set_shape_cuboid(
#				s.size * 0.5,
#				e.density,
#				e.entity_type,
#			)
#		else:
#			push_warning("Shape not handled")
#		c.free()
#	elif e.collision_shape is CollisionPolygon2D:
#		var c :CollisionPolygon2D = e.collision_shape
#		var p := Geometry2D.decompose_polygon_in_convex(c.get_polygon())
#		b.set_shape_polygons(
#				p,
#				e.density,
#				e.entity_type,
#			)
#		c.free()
#	else:
#		push_warning("No shape")
#
#	# Create the render scene
#	e.set_script(e.render_script)
#	var p := PackedScene.new()
#	p.pack(e)
#	b.set_render_scene(p)
#
#	b.build()
#	e.free()

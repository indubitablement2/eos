extends Client

func _notification(what: int) -> void:
	if what == NOTIFICATION_WM_CLOSE_REQUEST:
		clear_data()

func load_data() -> void:
	clear_data()
	
	# [[path, node]]
	var nodes = []
	var dirs :Array[String] = ["res:/"]
	while !dirs.is_empty():
		var current_path :String = dirs.pop_back() + "/"
		var access := DirAccess.open(current_path)
		access.list_dir_begin()
		var file_name := access.get_next()
		while file_name:
			var file_path := current_path + file_name
			if access.current_is_dir():
				dirs.push_back(file_path)
			else:
				var node := _try_load_path(file_path)
				if node:
					nodes.push_back([file_path, node])
			file_name = access.get_next()
	
	nodes.sort_custom(_sort_nodes)
	
	var entity_data := []
	var ship_data := []
	for node in nodes:
		if node[1] is EntityData:
			entity_data.push_back(node)
		else:
			ship_data.push_back(node)
	
	for e in entity_data:
		_build_entity_data(e[0], e[1])
	for s in ship_data:
		_build_ship_data(s[0], s[1])
	
	dbg_print_data()

func _try_load_path(path: String) -> Node:
	var ext := path.get_extension()
	if ext != "tscn" && ext != "scn":
		return null
	
	var res := load(path) as PackedScene
	if !res:
		return null
	if !res.can_instantiate():
		return null
	
	var node := res.instantiate()
	if node is ShipData || node is EntityData:
		return node
	else:
		node.free()
		return null

func _sort_nodes(a, b) -> bool:
	return a[0] < b[0]

func _build_entity_data(path: String, e: EntityData) -> void:
	var b := EntityDataBuilder.new()
	
	b.set_path(path)
	b.set_angular_acceleration(e.angular_acceleration)
	b.set_simulation_script(e.simulation_script)
	b.set_aproximate_radius(e.aproximate_radius)
	b.set_linear_acceleration(e.linear_acceleration)
	b.set_max_angular_velocity(e.max_angular_velocity)
	b.set_max_linear_velocity(e.max_linear_velocity)
	
	# Handle hulls
	var i := 0
	for child in e.get_children():
		var h := child as HullData
		if h:
			var hb := HullDataBuilder.new()
			hb.set_render_node_idx(i)
			_build_hull_data(hb, h)
			b.add_hull(hb)
		i += 1
	
	var p := PackedScene.new()
	p.pack(e)
	b.set_render_scene(p)
	
	b.build()
	e.free()

func _build_hull_data(b: HullDataBuilder, h: HullData):
	b.set_armor(h.armor)
	b.set_density(h.density)
	b.set_hull(h.hull)
	
	if h.collision_shape is CollisionShape2D:
		var c :CollisionShape2D = h.collision_shape
		if c.shape is CircleShape2D:
			var s : CircleShape2D = c.shape
			b.set_shape_circle(s.radius)
		elif c.shape is RectangleShape2D:
			var s : RectangleShape2D = c.shape
			b.set_shape_cuboid(s.size * 0.5)
	else:
		var c :CollisionPolygon2D = h.collision_shape
		b.set_shape_polygon(c.get_polygon())
	h.collision_shape.free()
	
	h.set_script(h.render_script)

func _build_ship_data(ship_path: String, s: ShipData) -> void:
	var b := ShipDataBuilder.new()
	
	b.set_path(ship_path)
	b.set_entity_data_path(s.entity_path)
	b.set_display_name(s.display_name)
	var texture := s.get_texture()
	if texture:
		b.set_texture(texture)
	
	b.build()
	s.free()

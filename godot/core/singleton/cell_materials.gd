extends Node

var cell_material_names: Array[StringName] = []

func _ready() -> void:
	load_cell_materials()

func load_cell_materials() -> void:
	var materials := _get_materials()
	
	cell_material_names.resize(materials.size())
	cell_material_names.fill(&"")
	
	Grid.init_materials(materials.size())
	var idx := 0
	for m in materials:
		cell_material_names[idx] = m.display_name
		
		Grid.add_material(
			m.movement_type,
			m.density,
			m.durability,
			m.collision_type,
			m.friction,
			m.reactions,
			idx
		)
		
		idx += 1
	
	print(cell_material_names)
#	Grid.print_materials()

func _get_materials() -> Array[CellMaterialData]:
	var materials: Array[CellMaterialData] = []
	var reactions: Array[CellReactionData] = []
	
	var dirs :Array[String] = ["res:/"]
	while !dirs.is_empty():
		var current_path :String = dirs.pop_back() + "/"
		var access := DirAccess.open(current_path)
		access.list_dir_begin()
		var file_name := access.get_next()
		while file_name:
			var file_path := current_path + file_name
			var is_dir = access.current_is_dir()
			file_name = access.get_next()
			if is_dir:
				dirs.push_back(file_path)
			else:
				var ext := file_path.get_extension()
				if ext != "tres" && ext != "res":
					continue
				
				var res := load(file_path)
				if res == null:
					continue
				
				if res is CellMaterialData:
					materials.push_back(res)
				elif res is CellReactionData:
					reactions.push_back(res)
	
	_demangle_reactions(materials, reactions)
	
	return materials

# Sort by their number of reaction 
# If eq, sort by id
static func _sort_mat(a: CellMaterialData, b: CellMaterialData) -> bool:
	if a.id.is_empty():
		return true
	elif b.id.is_empty():
		return false
	elif a.num_reaction == b.num_reaction:
		return a.id > b.id
	else:
		return a.num_reaction > b.num_reaction

# Add empty and make sure it is first
# Replace tags with id
# Remove reaction with missing id
# Sort materials by its number of reaction or id
# Add reactions to materials
# Remove duplicate materials
func _demangle_reactions(
	materials: Array[CellMaterialData],
	reactions: Array[CellReactionData],
) -> void:
	# Add "" material
	materials.push_back(CellMaterialData.new())
	
	# Add all tag to all cell materials
	for m in materials:
		m.tags.push_back("all")
	
	# {tag String : tagged_id Array[String]}
	var tags := {}
	
	# Get the tags
	for m in materials:
		for tag in m.tags:
			if !tags.has(tag):
				tags[tag] = []
			tags[tag].push_back(m.id)
	
	# Replace reaction tags with id
	var i := 0
	while i < reactions.size():
		var r := reactions[i]
		if tags.has(r.in1):
			reactions.remove_at(i)
			for id in tags[r.in1]:
				var new_r: CellReactionData = r.duplicate()
				new_r.in1 = id
				reactions.push_back(new_r)
		else:
			i += 1
	i = 0
	while i < reactions.size():
		var r := reactions[i]
		if tags.has(r.in2):
			reactions.remove_at(i)
			for id in tags[r.in2]:
				var new_r: CellReactionData = r.duplicate()
				new_r.in2 = id
				reactions.push_back(new_r)
		else:
			i += 1
	
	for r in reactions:
		if r.do_not_change_material1:
			r.out1 = r.in1
		if r.do_not_change_material2:
			r.out2 = r.in2
	
	# {id : int}
	var num_reaction := {}
	
	# Dedup materials and init reaction count
	i = 0
	while i < materials.size():
		var m = materials[i]
		if num_reaction.has(m.id):
			push_warning("cell material id is not inique: ", m.id)
			materials.remove_at(i)
			continue
		
		num_reaction[m.id] = 0
		i += 1
	
	# Remove reaction with missing id
	i = 0
	while i < reactions.size():
		var r = reactions[i]
		if !num_reaction.has(r.in1):
			push_warning(r.in1, " not found")
			reactions.remove_at(i)
		elif !num_reaction.has(r.in2):
			push_warning(r.in2, " not found")
			reactions.remove_at(i)
		elif !num_reaction.has(r.out1):
			push_warning(r.out1, " not found")
			reactions.remove_at(i)
		elif !num_reaction.has(r.out2):
			push_warning(r.out2, " not found")
			reactions.remove_at(i)
		else:
			num_reaction[r.in1] += 1
			num_reaction[r.in2] += 1
			i += 1
	
	for m in materials:
		m.num_reaction = num_reaction[m.id]
	
	# Sort material by reaction count
	materials.sort_custom(_sort_mat)
	assert(materials[0].id.is_empty())
	
	# Add reaction to materials
	var materials_idx := {}
	i = 0
	for m in materials:
		m.reactions.resize(materials.size() - i)
		for j in m.reactions.size():
			m.reactions[j] = []
		materials_idx[m.id] = i
		i += 1
	for r in reactions:
		var probability := int(r.probability * float(0xffffffff))
		
		var in1: int = materials_idx[r.in1]
		var in2: int = materials_idx[r.in2]
		var out1: int = materials_idx[r.out1]
		var out2: int = materials_idx[r.out2]
		
		if in1 > in2:
			materials[in2].reactions[in1].push_back([probability, out2, out1])
		else:
			materials[in1].reactions[in2].push_back([probability, out1, out2])



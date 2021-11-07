extends Node2D

onready var canvas_rid := get_canvas_item()
onready var mesh_rid := VisualServer.mesh_create()
onready var multimesh_rid := VisualServer.multimesh_create()

onready var viewport_size := get_tree().get_root().get_size()

onready var def := []
onready var def_img := Image.new()
onready var def_tex := ImageTexture.new()

onready var tex_rid := SpritePacker.tex.get_rid()

func _ready() -> void:
	pass
#	var gen_img := Image.new()
#	gen_img.load("res://assets/generation/galaxy_gen.png")
#	$Game.generate_world("test world", gen_img)
#	_init_mesh()
#	_init_mat()
#	allocate_mesh(100)

func _draw() -> void:
	# get data from currently displayed scape.
	VisualServer.canvas_item_add_multimesh(canvas_rid, multimesh_rid, tex_rid)

func _exit_tree() -> void:
	VisualServer.free_rid(multimesh_rid)
	VisualServer.free_rid(mesh_rid)

#func _process(delta: float) -> void:
#	if Input.is_mouse_button_pressed(1):
#		print("placing system at: " + str(get_global_mouse_position()))
#		$Game.new_system(get_global_mouse_position())

func allocate_mesh(num: int) -> void:
	VisualServer.multimesh_allocate(multimesh_rid, num, VisualServer.MULTIMESH_TRANSFORM_2D, VisualServer.MULTIMESH_COLOR_NONE, VisualServer.MULTIMESH_CUSTOM_DATA_FLOAT)

# Setup base mesh and multimesh.
func _init_mesh() -> void:
	var vertices := PoolVector2Array()
	vertices.push_back(Vector2(-0.5, -0.5))
	vertices.push_back(Vector2(0.5, -0.5))
	vertices.push_back(Vector2(0.5, 0.5))
	vertices.push_back(Vector2(-0.5, 0.5))
	
	var uvs := PoolVector2Array()
	uvs.push_back(Vector2(0, 0));
	uvs.push_back(Vector2(1, 0));
	uvs.push_back(Vector2(1, 1));
	uvs.push_back(Vector2(0, 1));
	
	var colors := PoolColorArray()
	colors.push_back(Color(1, 1, 1, 1));
	colors.push_back(Color(1, 1, 1, 1));
	colors.push_back(Color(1, 1, 1, 1));
	colors.push_back(Color(1, 1, 1, 1));
	
	var indices := PoolIntArray()
	indices.push_back(0);
	indices.push_back(1);
	indices.push_back(2);
	indices.push_back(2);
	indices.push_back(3);
	indices.push_back(0);
	
	var arr := []
	arr.resize(VisualServer.ARRAY_MAX)
	arr[VisualServer.ARRAY_VERTEX] = vertices
	arr[VisualServer.ARRAY_TEX_UV] = uvs
	arr[VisualServer.ARRAY_COLOR] = colors
	arr[VisualServer.ARRAY_INDEX] = indices
	
	VisualServer.mesh_add_surface_from_arrays(mesh_rid, VisualServer.PRIMITIVE_TRIANGLES, arr)
	VisualServer.multimesh_set_mesh(multimesh_rid, mesh_rid)

func _init_mat() -> void:
	get_material().set_shader_param("def_texture", def_tex)

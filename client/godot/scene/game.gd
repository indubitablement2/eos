extends Node2D

onready var canvas_rid := get_canvas_item()
onready var mesh_rid := VisualServer.mesh_create()
onready var multimesh_rid := VisualServer.multimesh_create()

onready var viewport_size := get_tree().get_root().get_size()

onready var def := []
onready var def_img := Image.new()
onready var def_tex := ImageTexture.new()

onready var tex_rid := SpritePacker.tex.get_rid()

onready var client := $Client

func _ready() -> void:
	$CanvasLayer/Debug/TimeDilation.connect("draw", client, "_on_draw_time_dilation", [$CanvasLayer/Debug/TimeDilation])	
	
	client.connect("ConnectionResult", self, "_on_connection_result")
#	var gen_img = preload("res://assets/debug/target.png").get_data()
#	var gen_img := Image.new()
#	gen_img.load("res://assets/generation/galaxy_gen.png")
#	$Game.generate_metascape("test world", 2000.0, preload("res://assets/debug/pixel.png").get_data())
	print(IP.get_local_addresses())
	
#	_init_mesh()
#	_init_mat()
#	allocate_mesh(100)

func _exit_tree() -> void:
	VisualServer.free_rid(multimesh_rid)
	VisualServer.free_rid(mesh_rid)

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


func _on_Button_pressed() -> void:
	var result = client.connect_to_server("::1", 2)
	print("Connection start result: " + str(result))
	if result:
		$Button.hide()

func _on_connection_result(result: bool) -> void:
	if result:
		$Button.hide()

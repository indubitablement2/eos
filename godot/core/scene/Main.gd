extends Control

const GRID_WIDTH = 64 + 32 * 7
const GRID_HEIGHT = 64 + 32 * 4

@onready var tex :ImageTexture
@onready var sp := Sprite2D.new()

func _ready() -> void:
#	Grid.run_tests()
	
	Grid.new_empty(GRID_WIDTH, GRID_HEIGHT)
	var grid_size := Grid.get_size()
	
	var img := Image.create(grid_size.x + 50, grid_size.y + 50, false, Image.FORMAT_RF)
	tex = ImageTexture.create_from_image(img)
	
	$render.add_child(sp)
	sp.centered = false
	sp.position = Vector2(-25.0, -25.0)
	sp.set_texture(tex)
	
	var mat := ShaderMaterial.new()
	mat.set_shader(preload("res://core/shader/cell.gdshader"))
	sp.set_material(mat)
	
#	Grid.update_texture_data(tex, Vector2i(0, 0))
	
#	print("grid size: ", Grid.get_size())
#	print("grid size chunk: ", Grid.get_size_chunk())

#func _unhandled_input(event: InputEvent) -> void:
#	if event.is_action_pressed("down"):
#		Grid.step_manual()
#		Grid.update_texture_data(tex, Vector2i(0, 0))

func _process(_delta: float) -> void:
#	if Input.is_action_pressed("up"):
	Grid.step_manual()
	Grid.update_texture_data(tex, Vector2i(sp.position))
	
	var mouse_pos := get_global_mouse_position() / Grid.GRID_SCALE
	var grid_pos := Vector2i(mouse_pos)
	var mat_idx := Grid.get_cell_material_idx(grid_pos)
	
	$CellName.set_text(CellMaterials.cell_material_names[mat_idx])
	$Tick.set_text(str(Grid.get_tick()))
	$ChunkActive.set_text(str(Grid.is_chunk_active(Vector2i(1, 1))))
	$GridPosition.set_text(str(grid_pos))


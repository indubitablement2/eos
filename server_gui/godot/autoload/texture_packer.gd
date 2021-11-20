extends Node

# Terrain is an atlast of 8x8(64) textures. Each texture is 512x512 RGB8.
onready var terrain_material := load("res://shaders/terrain.material") as ShaderMaterial


func _ready() -> void:
	pack_terrain()


# Take all files inside res://assets/terrain/ that are 512x512 pixels, .png and RGB8.
# Combine them into an atlas.
func pack_terrain() -> void:
	# Data of the final image.
	var image_data := PoolByteArray([])
	# Array of PoolByteArray for each valid image found.
	var textures_datas := []
	
	# Get the data of each texture in terrain folder.
	var dir := Directory.new()
	if dir.open("res://assets/terrain/") == OK:
		if dir.list_dir_begin() != OK:
			push_error("Can not list dir.")
			return
		var file_name := dir.get_next()
		while file_name != "":
			if !dir.current_is_dir() and file_name.ends_with(".png"):
				var image := load("res://assets/terrain/" + file_name).get_data() as Image
				if image.get_size() == Vector2(512.0, 512.0) and image.get_format() == Image.FORMAT_RGB8:
					textures_datas.push_back(image.get_data())
			file_name = dir.get_next()
	else:
		push_error("An error occurred when trying to access the path.")
	
	var filler_array := PoolByteArray([])
	filler_array.resize(1536)
	
	# Merge the data. Fallback to filler_array if out of data.
	for y in textures_datas.size() / 8 + 1:
		for slice in 512:
			for x in 8:
				if textures_datas.size() <= x + y * 8:
					image_data.append_array(filler_array)
				else:
					image_data.append_array(textures_datas[x + 8 * y].subarray(slice * 1536, slice * 1536 + 1535))
	
	# Fill the rest of the image with 0.
	image_data.resize(50331648)
	
	# Convert to texture.
	var image := Image.new()
	image.create_from_data(8 * 512, 8 * 512, false, Image.FORMAT_RGB8, image_data)
	var terrain_texture := ImageTexture.new()
	terrain_texture.create_from_image(image, 0)
	
	# Set material param.
	terrain_material.set_shader_param("terrain_sampler", terrain_texture)

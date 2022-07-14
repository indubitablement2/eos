extends Node

onready var tex := preload("res://assets/debug/sheet_test.png")
#onready var tex_rid := tex.get_rid()
onready var tex_size := tex.get_size()

onready var def := []
onready var def_img := Image.new()
onready var def_tex := ImageTexture.new()

func _ready() -> void:
	# Make def texture.
	def.push_back(Rect2(0.0, 0.0, 64.0 / tex_size.x, 64.0 / tex_size.y))
	def.push_back(Rect2(64.0 / tex_size.x, 0.0, 64.0 / tex_size.x, 64.0 / tex_size.y))
	def.push_back(Rect2(128.0 / tex_size.x, 0.0, 64.0 / tex_size.x, 128.0 / tex_size.y))
	def.push_back(Rect2(0.0, 64.0 / tex_size.y, 128.0 / tex_size.x, 64.0 / tex_size.y))
	
	def_img.create(2, 2, false, Image.FORMAT_RGBAF)
	def_img.lock()
	for i in 4:
		def_img.set_pixel(i % 2, i / 2, Color(def[i].position.x, def[i].position.y, def[i].end.x, def[i].end.y))
	def_img.unlock()
	def_tex.create_from_image(def_img, 0)

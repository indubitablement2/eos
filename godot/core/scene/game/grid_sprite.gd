extends Sprite2D

@onready var img_tex := texture

func _ready() -> void:
	var img := Image.create(1, 1, false, Image.FORMAT_RF)
	(texture as ImageTexture).set_image(img)

func _process(delta: float) -> void:
	if Grid.get_size().x == 0:
		return
	
	var rect := _view_rect()
	
	if rect.size.x > texture.get_width() || rect.size.y > texture.get_height():
		_resize_texture(rect.size)
	
	position = rect.position
	Grid.update_texture_data(texture as ImageTexture, rect.position)

func _resize_texture(texture_size: Vector2i) -> void:
	texture_size.x = nearest_po2(texture_size.x)
	texture_size.y = nearest_po2(texture_size.y)
	(texture as ImageTexture).set_size_override(texture_size)

func _view_rect() -> Rect2i:
	var ctrans := get_canvas_transform()
	var view_origin := -ctrans.get_origin() / ctrans.get_scale()
	var view_size := get_viewport_rect().size / ctrans.get_scale()
	
	return Rect2i(view_origin, view_size + Vector2.ONE)

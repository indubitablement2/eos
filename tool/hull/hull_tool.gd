@tool
extends HullServer


@export var sprite : Sprite2D


func _init() -> void:
	pass

func _enter_tree() -> void:
	pass

func _exit_tree() -> void:
	pass

func _process(_delta: float) -> void:
	queue_redraw()

func _physics_process(_delta: float) -> void:
	pass

func _integrate_forces(_state: PhysicsDirectBodyState2D) -> void:
	pass

func _draw() -> void:
	if sprite && data:
		if sprite.texture && data.armor_max_relative_texture:
			var pos := sprite.position
			if sprite.centered:
				pos -= sprite.texture.get_size() * 0.5
			
			var armor_size := Vector2(compute_armor_size()) * ARMOR_SCALE
			draw_set_transform(
				pos,
				0.0,
				armor_size / data.armor_max_relative_texture.get_size())
			draw_texture(
				data.armor_max_relative_texture,
				Vector2.ZERO,
				Color(1.0, 1.0, 1.0, 0.25))
			draw_set_transform(Vector2.ZERO)
	
	if data:
		draw_arc(
			Vector2.ZERO,
			data.radius, 0.0,
			INF,
			maxi(int(data.radius / 3.0), 3),
			Color.WHITE,
			1.0,
			true)


func compute_armor_size() -> Vector2i:
	if !sprite.texture:
		return Vector2i(3, 3)
	var armor_size := Vector2i((sprite.texture.get_size() / Hull.ARMOR_SCALE).ceil())
	armor_size = armor_size.clamp(Hull.ARMOR_SIZE_MIN, Vector2(128, 128))
	return armor_size


#if armor_relative_texture == null:
	#armor_relative_texture = Util.PIXEL_TEXTURE
	
	#armor_relative_image = armor_relative_texture.get_image()
	#armor_relative_image.convert(Image.FORMAT_RH)
	#var armor_size := compute_armor_size(sprite)
	#armor_relative_image.resize(armor_size.x, armor_size.y)



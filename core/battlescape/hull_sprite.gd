extends Sprite2D
class_name HullSprite


const HULL_SHADER := preload("res://core/shader/hull.gdshader")


## Multiply recent hull damage
const RECENT_DAMAGE_MUTIPLIER := 10.0

## How long does it take for recent damage to clear up.
const RECENT_DAMAGE_CLEAR_TIME := 4.0

var recent_damage_image : Image
var recent_damage_texture_dirty := true
var recent_damage_texture : ImageTexture


var armor_relative_texture_dirty := true
var armor_relative_texture : ImageTexture


## Time since last taken damage. 
var damage_last := 256.0


@onready var hull : Hull = get_parent()


func _ready() -> void:
	armor_relative_texture = ImageTexture.create_from_image(hull.armor_relative_image)
	
	recent_damage_image = Image.create(
		hull.armor_relative_image.get_width(),
		hull.armor_relative_image.get_height(),
		false,
		Image.FORMAT_RH)
	
	recent_damage_texture = ImageTexture.create_from_image(recent_damage_image)
	
	var mat := ShaderMaterial.new()
	mat.set_shader(HULL_SHADER)
	mat.set_shader_parameter("armor_max_relative_texture", hull.data.armor_relative_texture)
	mat.set_shader_parameter("armor_relative_texture", armor_relative_texture)
	mat.set_shader_parameter("recent_damage_texture", recent_damage_texture)
	material = mat


func _process(delta: float) -> void:
	damage_last += delta
	
	# TODO: Only if we can see sprite
	if true:
		if recent_damage_texture_dirty:
			if damage_last < RECENT_DAMAGE_CLEAR_TIME:
				recent_damage_image.adjust_bcs(
					RECENT_DAMAGE_CLEAR_TIME / RECENT_DAMAGE_CLEAR_TIME - damage_last, 1.0, 1.0)
			else:
				recent_damage_texture_dirty = false
				recent_damage_image.fill(Color.BLACK)
			recent_damage_texture.update(recent_damage_image)
		
		if armor_relative_texture_dirty:
			armor_relative_texture_dirty = false
			armor_relative_texture.update(hull.armor_relative_image)


func took_hull_damage(amount: float, cell_center: Vector2i) -> void:
	amount *= RECENT_DAMAGE_MUTIPLIER / hull.hull_hp
	var i := 0
	for cell_offset in Hull.ARMOR_CELL_OFFSET:
		var point_cell := cell_center + cell_offset
		var cell := recent_damage_image.get_pixelv(point_cell).r
		cell += amount * Hull.ARMOR_CELL_EFFECT[i]
		cell = minf(cell, 1.0)
		recent_damage_image.set_pixelv(point_cell, Color(cell, 0.0, 0.0))
		i += 1


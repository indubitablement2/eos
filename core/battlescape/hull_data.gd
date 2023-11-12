@tool
extends Resource
class_name HullData

static var HULL_CLASS := PackedStringArray([
	"missile",
	"fighter",
	"frigate",
	"destroyer",
	"cruiser",
	"battleship",
	"experimental"])
enum HullClass {
	MISSILE,
	FIGHTER,
	FRIGATE,
	DESTROYER,
	CRUISER,
	BATTLESHIP,
	EXPERIMENTAL}
@export
var hull_class := HullClass.MISSILE


@export
var sprite: Texture2D = Util.PIXEL_TEXTURE
@export
var sprite_offset := Vector2.ZERO


@export_range(1.0, 500.0, 0.5, "or_greater")
var radius := 10.0


@export_range(0.0, 2000.0, 10.0, "or_greater", "suffix:px/sec^2")
var linear_acceleration := 800.0
@export_range(0.0, 2000.0, 10.0, "or_greater", "suffix:px/sec")
var linear_velocity_max := 400.0
@export_range(0.0, 20.0, 0.1, "or_greater", "suffix:rad/sec^2")
var angular_acceleration := 8.0
@export_range(0.0, 20.0, 0.1, "or_greater", "suffix:rad/sec")
var angular_velocity_max := 4.0


@export_range(0.01, 4.0, 0.01, "or_greater")
var time_scale := 1.0


@export_range(1.0, 10000.0, 1.0, "or_greater")
var hull_hp_max := 100.0

@export_range(1.0, 2000.0, 1.0, "or_greater")
var armor_hp_max := 1.0
@export_range(0.0, 4.0, 0.01, "or_greater")
var armor_min_effectiveness := 0.1

@export 
var armor_relative_texture : Texture2D = Util.PIXEL_TEXTURE


## Do not force hull material.
## Do not track recent damage.
## Use armor_hp_max as current armor instead of armor cells. 
## Intended for simple missile.
@export
var simplified_hull := false


@export_group("Computed")
@export
var armor_relative_image : Image


func is_ship() -> bool:
	return hull_class >= HullClass.FRIGATE


func _verify() -> void:
	if armor_relative_texture == null:
		armor_relative_texture = Util.PIXEL_TEXTURE
	
	var armor_size := Vector2i((sprite.get_size() / Hull.ARMOR_SCALE).ceil())
	armor_size = armor_size.clamp(Hull.ARMOR_SIZE_MIN, Vector2(128, 128))
	
	armor_relative_image = armor_relative_texture.get_image()
	armor_relative_image.convert(Image.FORMAT_RH)
	armor_relative_image.resize(armor_size.x, armor_size.y)



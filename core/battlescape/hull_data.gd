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
@export var hull_class := HullClass.MISSILE


@export var sprite: Texture2D
## Offset to render sprite and armor grid.
@export var offset := Vector2.ZERO


@export_range(1.0, 500.0, 0.5, "or_greater") var radius := 10.0


@export_group("Movement")
@export_range(0.0, 2000.0, 10.0, "or_greater", "suffix:px/sec^2")
var linear_acceleration := 800.0
@export_range(0.0, 2000.0, 10.0, "or_greater", "suffix:px/sec")
var linear_velocity_max := 400.0
@export_range(0.0, 20.0, 0.1, "or_greater", "suffix:rad/sec^2")
var angular_acceleration := 8.0
@export_range(0.0, 20.0, 0.1, "or_greater", "suffix:rad/sec")
var angular_velocity_max := 4.0


@export_group("Defence")
@export_range(1.0, 10000.0, 1.0, "or_greater") var hull_hp_max := 100.0
## Current hull hp. Hull is destroyed when this reach 0.
@export var hull_hp := 100.0

## How much armor does a cell with a value of 1 have.
@export_range(0.0, 2000.0, 1.0, "or_greater")
var armor_hp_max := 0.0
## Current armor value for each cells relative ([0..1]) to armor_hp_max.
## Always a minimum of 3x3.
## FORMAT_RH
## Computed automatically from armor_max_relative_texture.
@export var armor_relative_image : Image
## Maximum armor value for each cells relative ([0..1]) to armor_hp_max. 
## FORMAT_RH
## Computed automatically from armor_max_relative_texture.
@export var armor_max_relative_image : Image
## Used so that shader know what value is full armor.
## Can be any size. Will be stretch to fit.
@export var armor_max_relative_texture : Texture2D = Util.PIXEL_TEXTURE

## Multiply armor effect.
@export var armor_effectiveness := 1.0
## Armor will keep at least this much effect even when no armor remain.
@export_range(0.0, 1.0, 0.01, "or_greater") var armor_min_effectiveness := 0.1

# TODO: Damage type received mutiplier


# TODO: Turret slots


@export_group("Turret Modifier")
## ballistic, missile, energy
## Range multiplier.
## What this does specifically is left to be interpreted by the turret.
@export var turret_range := Vector3.ONE
@export var damage_mutiplier := Vector3.ONE
@export var ammo_replenish_delay := Vector3.ONE
@export var rotation_speed := Vector3.ONE
@export var fire_delay := Vector3.ONE
@export var ammo_max := Vector3.ONE

### hull, armor, shield, emp
#@export var damage_type_mutiplier := Vector4.ONE

## Helps predict target's movement. Higher is more accurate.
@export_range(0, 4, 1) var prediction_iter := 0


func is_ship() -> bool:
	return hull_class >= HullClass.FRIGATE


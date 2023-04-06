extends Resource
class_name CellMaterialData

@export var display_name: StringName

@export var id: String
@export var tags: PackedStringArray

@export var color := Color.TRANSPARENT

@export_category("Movement")
@export var movement_type := Grid.CELL_MOVEMENT_SOLID
#@export_range(0.1, 1.0) var movement_speed := 1.0
@export var density := 0

@export_category("Interaction")
@export var durability := 0

@export_category("Collision")
@export var collision_type := Grid.CELL_COLLISION_NONE
## Less than 1 is slippery, more than 1 is sticky.
@export_range(0.0, 2.0, 0.01, "or_greater") var friction := 1.0
#@export_range(0.0, 0.9, 0.01) var bounciness := 0.0

@export_category("Ignite Preset")
@export var use_fire_preset := false
## Each level higher than `ignite_temperature_needed`
## will have 2.5x the chance to ignite.
## With ignite_temperature_needed set to Amber and  ignite_chance to 0.05:
## Amber: 0.1, Fire: 0.25, RagingFire: 0.625
@export var ignite_chance := 0.1
@export_enum("Amber", "Fire", "RagingFire") var ignite_temperature_needed := 0
@export_enum("Air", "Amber") var ignite_into_preset :String
@export var ignite_into_custom :String

@export_category("Event")
# Chance/do what?
@export var on_destroyed := false

var num_reaction := 0
## [[probability: int, out1: int, out2: int]]
var reactions := []

# TODO:
# spawn texture
# mine item
# damage
# grow vegetation
# electricity
# ai?
# audio
# explosion?
# stain

func _to_string() -> String:
	return id + " - " + display_name + ": " + str(reactions)
	

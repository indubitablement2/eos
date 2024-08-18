extends CollisionPolygon2D
class_name EntityArmor

@export var entity_hp_max := 100.0
@onready var entity_hp := entity_hp_max

@export var armor_hp_max := 0.0
@export var armor_min_effectiveness := 0.1
@onready var armor_hp := armor_hp_max

@export var engine_arc := 0.1

@onready var entity: Entity = get_parent()

## global_point is where damage originated.
## multipliers:
## x: entity
## y: armor
## z: shield
## w: emp
func damage(
	amount: float,
	global_point: Vector2,
	damage_multiplier: Vector4) -> void:
	print(amount, global_point, damage_multiplier)

func destroy() -> void:
	entity.destroy()

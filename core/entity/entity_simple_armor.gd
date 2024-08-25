extends CollisionShape2D
class_name EntitySimpleArmor

@export var entity_hp_max := 100.0
@onready var entity_hp := entity_hp_max

@export var armor_hp_max := 0.0
@export var armor_min_effectiveness := 0.1
@onready var armor_hp := armor_hp_max

@onready var entity: Entity = get_parent()

## global_point is where damage originated.
## multipliers:
## x: entity
## y: armor
## z: shield
## w: emp
func damage(amount: float, global_point: Vector2, multipliers: Vector4) -> Vector4:
	var armor_dmg := amount * multipliers.y
	
	# Comoute damage reduction from armor.
	var effective_armor := maxf(armor_hp, armor_hp_max * armor_min_effectiveness)
	var dmg_reduction := armor_dmg / (armor_dmg + effective_armor)
	
	# Armor damage.
	armor_dmg *= dmg_reduction
	armor_dmg = minf(armor_dmg, armor_hp)
	armor_hp -= armor_dmg
	
	# Entity damage.
	var entity_dmg := amount * dmg_reduction * multipliers.x
	entity_hp -= entity_dmg
	
	# Engine damage.
	if PI - absf(get_angle_to(global_point)) < entity.engine_arc:
		entity.engine_hp -= entity_dmg / entity_hp_max * entity.engine_hp_relative_max
	
	# Emp damage.
	# Simple armor only have engine and are expected to be small.
	# Therefore we can assume hit was close to the engine.
	var emp_dmg := amount * multipliers.w
	entity.engine_hp -= emp_dmg / entity_hp_max * entity.engine_hp_relative_max
	
	if entity_hp < 1.0:
		destroy()
	
	return Vector4(entity_dmg, armor_dmg, 0.0, emp_dmg)

func destroy() -> void:
	entity.destroyed.emit()

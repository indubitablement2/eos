extends CollisionShape2D
class_name HullSimpleArmor

@export var hull_hp_max := 100.0
@onready var hull_hp := hull_hp_max

@export var armor_hp_max := 0.0
@export var armor_min_effectiveness := 0.1
@onready var armor_hp := armor_hp_max

@onready var hull: Hull = get_parent()

## global_point is where damage originated.
## multipliers:
## x: hull
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
	
	# Hull damage.
	var hull_dmg := amount * dmg_reduction * multipliers.x
	hull_hp -= hull_dmg
	
	# Engine damage.
	if PI - absf(get_angle_to(global_point)) < hull.engine_arc:
		hull.engine_hp -= hull_dmg / hull_hp_max * hull.engine_hp_relative_max
	
	# Emp damage.
	# Simple armor only have engine and are expected to be small.
	# Therefore we can assume hit was close to the engine.
	var emp_dmg := amount * multipliers.w
	hull.engine_hp -= emp_dmg / hull_hp_max * hull.engine_hp_relative_max
	
	if hull_hp < 1.0:
		destroy()
	
	return Vector4(hull_dmg, armor_dmg, 0.0, emp_dmg)

func destroy() -> void:
	hull.destroy()

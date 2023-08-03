extends EntityBase
class_name Entity


enum EntityCollisionType {
	SHIP = 1 | 2 | 3,
	BANANA = 1 | 5 | 3,
}


@export var hull_srpite : HullSprite
@export var turrets : Array[Turret]
@export var entity_collision_type := EntityCollisionType.SHIP


var player_controlled := false


func _integrate_forces(state: PhysicsDirectBodyState2D) -> void:
	# Contacts
	var contact_count := state.get_contact_count()
	var contact_idx := 0
	while contact_idx < contact_count:
		var contact_impulse := state.get_contact_impulse(contact_idx)
		print(contact_impulse)
	
	_base_integrate_forces(state)
	

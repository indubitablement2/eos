extends EntityBase


enum EntityCollisionType {
	SHIP = 1 | 2 | 3,
	BANANA = 1 | 5 | 3,
}


@export var hull_srpite : HullSprite
@export var turret_slots : Array[TurretSlot] = []
@export var entity_collision_type := EntityCollisionType.SHIP


var turrets_primary : Array[Turret] = []
var turrets_secondary : Array[Turret] = []
var turrets_auto : Array[Turret] = []


var player_controlled := false
## INF used as a flag for turrets to take their default rotation.
var aim_at := Vector2.INF
## 0: none
## 1..14: just pressed actions (respective auto only flags also on)
## 14..28 auto only actions
var actions := 0


#func _enter_tree() -> void:
#	update_turret_groups()


func _integrate_forces(state: PhysicsDirectBodyState2D) -> void:
#	# Contacts
#	var contact_count := state.get_contact_count()
#	var contact_idx := 0
#	while contact_idx < contact_count:
#		var contact_impulse := state.get_contact_impulse(contact_idx)
#
#		if !contact_impulse.is_zero_approx():
#			print(contact_impulse)
#
#		contact_idx += 1
	
	_base_integrate_forces(state)


#func update_turret_groups() -> void:
#	turrets_primary.clear()
#	turrets_secondary.clear()
#	turrets_auto.clear()
#
#	for turret_slot in turret_slots:
#		# TurretSlot only have a Turret child or nothing.
#		var turret := turret_slot.get_child(0)
#		if turret:
#			match turret_slot.turret_group:
#				TurretSlot.TurretGroup.PRIMARY:
#					turrets_primary.push_back(turret)
#				TurretSlot.TurretGroup.SECONDARY:
#					turrets_secondary.push_back(turret)
#				TurretSlot.TurretGroup.AUTO:
#					turrets_auto.push_back(turret)


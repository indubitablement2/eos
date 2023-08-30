extends Node2D
class_name DummyProjectile


## How long for a dummy to be back online (if we have ammo for it).
@export var cooldown := 0.5


@export_group("Save")
@export var cooldown_remaining := 0.0


func _enter_tree() -> void:
	# Parent should be a TurretDummyProjectile
	if cooldown_remaining <= 0.0:
		get_parent().dummies.push_back(self)
	else:
		get_parent().despawned.push_back(self)


@warning_ignore("unused_parameter")
func fire(from: Turret) -> void:
	remove()
	cooldown_remaining = cooldown

func remove() -> void:
	hide()

func reload() -> void:
	show()

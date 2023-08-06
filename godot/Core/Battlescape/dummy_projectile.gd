extends Node2D
class_name DummyProjectile


## How long for a dummy to be back online (if we have ammo for it).
@export var cooldown := 0.5


@export_group("Save")
@export var cooldown_remaining := 0.0


@warning_ignore("unused_parameter")
func fire(from: Turret) -> void:
	hide()


func reload() -> void:
	show()

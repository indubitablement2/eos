extends Resource
class_name TurretData


enum TurretType {
	Projectile,
	Missile}
@export
var turret_type := TurretType.Projectile


@export_range(0.0, 20.0, 0.1, "or_greater")
var rotation_speed := 4.0

@export_range(0.0, 10.0, 0.01, "or_greater")
var fire_delay := 0.5

@export_range(1, 500, 1, "or_greater")
var ammo_max := 1000000
@export_range(0.1, 10.0, 0.1, "or_greater")
var ammo_replenish_delay := INF
@export_range(1, 10, 1, "or_greater")
var ammo_replenish_amount := 1


@export_category("Ai")
## How much off target can this turret be to consider firing.
@export_range(0.001, PI, 0.001)
var effective_angle := 0.1

@export_range(1.0, 1000.0, 1.0, "or_greater")
var effective_range := 500.0

## If false, target missile and fighter.
@export
var target_ship := false

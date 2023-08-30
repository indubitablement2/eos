extends Resource
class_name TurretData


## How much off target can this turret be to consider firing.
@export_range(0.001, PI, 0.001)
var effective_angle := 0.1

@export_range(1.0, 1000.0, 1.0, "or_greater")
var effective_range := 500.0


@export_category("Ai")
## If false, target missile and fighter.
@export
var target_ship := false

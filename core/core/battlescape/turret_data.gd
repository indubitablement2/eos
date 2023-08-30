extends Resource
class_name TurretData


## How much off target can this turret be to consider firing.
@export_range(0.001, PI, 0.001)
var effective_angle := 0.01

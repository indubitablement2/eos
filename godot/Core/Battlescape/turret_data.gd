extends Resource
class_name TurretData


@export_range(0.0, 2000.0) var effective_range := 500.0
@export var rotation_speed := 8.0

## Delay between fire.
@export var fire_delay := 0.1

## false require just_pressed input to fire.
## Otherwise can hold input to fire continuously.
## Not used by ai.
@export var auto_fire := true


@export var max_ammo := INF
## How many ammo refilled per seconds.
@export var ammo_replenish_rate := 0.0


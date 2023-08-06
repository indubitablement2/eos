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


@export var max_ammo := 1000000
## How long to refille one ammo.
@export var ammo_replenish_delay := INF
@export var ammo_replenish_amount := 1


## How on target does this turret need to be to consider firing.
@export var auto_fire_angle_threshold := 0.05


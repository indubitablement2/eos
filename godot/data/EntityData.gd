class_name EntityData extends Node2D

## Optional script used for the simulation.
## Needs to extends EntityScript and be fully deterministic.
@export var simulation_script: GDScript
## Optional script used for rendering.
## Will replace data script with this one.
## Otherwise data script is simply removed.
@export var render_script: GDScript

@export_subgroup("Mobility")
@export var linear_acceleration := 32.0
@export var angular_acceleration := 16.0
@export var max_linear_velocity := 256.0
@export var max_angular_velocity := 64.0

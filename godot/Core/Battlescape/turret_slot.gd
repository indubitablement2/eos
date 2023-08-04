extends Node2D
class_name TurretSlot

enum TurretGroup {
	AUTO = 0,
	PRIMARY = 1,
	SECONDARY = 2,
}

enum TurretWeight {
	LIGHT,
	MEDIUM,
	HEAVY,
}

@export var max_weight := TurretWeight.LIGHT
## If set, turret can not be removed/replaced.
@export var builtin_turret := false

@export var turret_group := TurretGroup.PRIMARY

## If >= PI, can rotate without blocking.
@export_range(0.0, PI) var firing_arc := PI


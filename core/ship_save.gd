extends Resource
class_name ShipSave

@export var entity_scene: PackedScene

@export var hull := 1.0
@export var armor := 1.0
@export var modifiers: Array[PackedScene] = []
@export var turrets: Array[bool] = []

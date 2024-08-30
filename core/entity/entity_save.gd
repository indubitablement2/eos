extends Resource
class_name EntitySave

@export var data: EntityData

@export var hull := 1.0
@export var armor := 1.0
@export var modifiers: Array[PackedScene] = []
@export var turrets: Array[bool] = []

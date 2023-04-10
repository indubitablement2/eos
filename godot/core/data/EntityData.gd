extends Resource
class_name EntityData

@export var Id = ""

@export_category("Movement")
@export var LinearAceleration = 0.0
@export var AngularAcceleration = 0.0
@export var MaxLinearVelocity = 0.0
@export var MaxAngularVelocity = 0.0

@export_category("Defence")
@export var Readiness = 0.0
@export var HullHp = 0.0
@export var ArmorHp = 0.0

signal is_entity_data

extends Resource
class_name CellReactionData

@export_category("Inputs")

## Can be either a id or a tag.
## all is a special tag which is applied to all cell material.
@export var in1: String
## Can be either a id or a tag.
## all is a special tag which is applied to all cell material.
@export var in2: String

@export_category("Conditions")

@export_range(0.000001, 1.0, 0.000001) var probability: float = 1.0
#@export_flags("NW", "N", "NE", "W", "E", "SW", "S", "SE")
#var material2_offset := 0xff
#@export_group("Custom Value 1")
#@export_enum("None", "less_than", "more_than") var compare_custom_value_1 := 0
#@export var custom_value_1 := 0
#@export_group("Custom Value 2")
#@export_enum("None", "less_than", "more_than") var compare_custom_value_2 := 0
#@export var custom_value_2 := 0

@export_category("Outputs")

## If true, after reacting in1 will be unchanged.
@export var do_not_change_material1 := false
## A CellMaterialData id.
@export var out1: String
#@export var out_material1_custom_value_change: int
#@export var out_material1_shade_change: int

## If true, after reacting in2 will be unchanged.
@export var do_not_change_material2 := false
## A CellMaterialData id
@export var out2: String
#@export var out_material2_custom_value_change: int
#@export var out_material2_shade_change: int

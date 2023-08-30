@tool
extends Node
class_name EntityTool

enum ToolType {
	NONE,
	Armor,
}

@export var tool := ToolType.NONE : set = set_tool

func set_tool(value: ToolType) -> void:
	print(value)
	
	match value:
		ToolType.NONE:
			pass





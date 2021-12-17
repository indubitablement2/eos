extends Node


func _ready() -> void:
	pass


func _on_Generate_pressed() -> void:
	$SystemEditor.generate()
	$SystemEditor.update()

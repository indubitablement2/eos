extends Resource
class_name ShipData

## All ship should have one of these in res://you_mod_folder/ship_data/

@export var display_name: String
@export_multiline var description: String
@export var entity_scene: PackedScene
## Leaving null will take the texture from a node named Sprite2D from the entity scene.
@export var display_sprite: Texture2D


func _verify() -> void:
	if !display_sprite:
		var entity := entity_scene.instantiate()
		display_sprite = entity.get_node("Sprite2D").texture
		entity.queue_free()

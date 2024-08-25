extends Resource
class_name ShipData

@export var display_name: String
@export_multiline var description: String
@export var scene: PackedScene
## Leaving null will take the texture from a node named Sprite2D from the entity scene.
@export var display_sprite: Texture2D

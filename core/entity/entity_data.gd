extends Resource
class_name EntityData

enum EntityType {
	SHIP,
	FIGHTER,
	MISSILE,
	DEBRIS,
	PROJECTILE,
}
@export var entity_type := EntityType.SHIP
var entity_scene: PackedScene
## Leaving null will take the texture from a node named Sprite2D from the entity scene.
@export var display_sprite: Texture2D
## The distance between the center and the furthest armor edge.
var armor_size: float

@export var display_name: String
@export_multiline var description: String

func _verify(scn: PackedScene, entity: Entity) -> void:
	entity_scene = scn
	
	if !display_sprite:
		display_sprite = entity.get_node("Sprite2D").texture
	
	for child in entity.get_children():
		if child is EntityArmor:
			for point in child.polygon:
				armor_size = maxf((point + child.position).length(), armor_size)
		elif child is EntitySimpleArmor:
			armor_size = maxf((child.shape.get_rect().position + child.position).length(), armor_size)
			armor_size = maxf((child.shape.get_rect().end + child.position).length(), armor_size)

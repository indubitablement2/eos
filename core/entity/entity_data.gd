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
## Leaving null will take the texture from a node named Sprite2D from the entity scene.
@export var display_sprite: Texture2D

@export_group("Ship")
@export var display_name: String
@export_multiline var description: String

@export_group("ShipAI", "ship_ai_")
@export_flags_2d_physics var ship_ai_avoidance_mask: int

var entity_scene: PackedScene

func _verify(scn: PackedScene, entity: Entity) -> void:
	entity_scene = scn
	
	if !display_sprite:
		display_sprite = entity.get_node("Sprite2D").texture

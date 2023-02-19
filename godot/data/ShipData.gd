class_name ShipData extends Sprite2D

## Name as shown in-game.
@export var display_name := ""
## The entity spawned in battle.
@export_file("*.tscn", "*.scn") var entity_path :String

# Needed to identify ship data when scanning the project folder.
func _is_ship_data():
	pass

@tool
class_name EntityData extends Sprite2D

enum ENTITY_TYPE {
	Ship,
}

## Optional script used for the simulation.
## Needs to extends EntityScript and be fully deterministic.
@export var simulation_script: GDScript
## Optional script used for rendering.
## Will replace data script with this one.
## Otherwise data script is simply removed.
@export var render_script: GDScript

@export var entity_type: ENTITY_TYPE = ENTITY_TYPE.Ship
## Optional. If this entity is not a ship, leave this empty. 
@export var ship_data: ShipData : set = set_ship_data

@export_subgroup("Mobility")
@export var linear_acceleration := 32.0
## How fast this can accelerate in radian/seconds.
@export var angular_acceleration := 0.1
@export var max_linear_velocity := 256.0
## How fast this move in radian/seconds. 
## Something pushing it may cause higher speed.
@export var max_angular_velocity := 1.0

@export_subgroup("Defence")
## Maximum hull hp for the whole entity.
@export var hull := 100
## Maximum armor hp of individual partitions(squares).
##
## Material of this Srpite2D will be overridden to display damage.
@export var armor := 100

@export_subgroup("Physic")
## Node will not exist in-game.
##
## Only support CollisionPolygon2D(circle, rectangle)
## and CollisionShape2D(only used for the point array).
## 
## This act as the entity center, so if the sprite needs to be offset, 
## you change the position/rotation of this.
@export var collision_shape: Node2D
@export var density := 1.0

@export_subgroup("Rendering")
## An aproximate radius for this entity. 
## Used for rendering and selection.
@export var aproximate_radius := 64.0 : set = set_aproximate_radius
@export var hide_aproximate_radius_preview := true : set = set_hide_aproximate_radius_preview
@export var hide_target_preview := false : set = set_hide_target_preview

func _draw() -> void:
	draw_set_transform(-position, PI * 0.25 - rotation)
	if !hide_aproximate_radius_preview:
		draw_circle(Vector2.ZERO, aproximate_radius, Color(1.0, 1.0, 1.0, 0.1))
	if !hide_target_preview:
		draw_rect(Rect2(-aproximate_radius, -aproximate_radius, aproximate_radius * 2.0, aproximate_radius * 2.0), Color.RED, false, 4.0)

func set_ship_data(value: ShipData) -> void:
	if value:
		entity_type = ENTITY_TYPE.Ship
	ship_data = value

func set_aproximate_radius(value: float) -> void:
	aproximate_radius = value
	queue_redraw()

func set_hide_aproximate_radius_preview(value: bool) -> void:
	hide_aproximate_radius_preview = value
	queue_redraw()

func set_hide_target_preview(value: bool) -> void:
	hide_target_preview = value
	queue_redraw()

#func center_collision_polygon(_value) -> void:
#	var c = collision_shape as CollisionPolygon2D
#	if c:
#		for i in c.polygon.size():
#			c.polygon[i] = c.polygon[i] * c.transform.inverse()
#		c.position = Vector2.ZERO
#		c.rotation = 0.0

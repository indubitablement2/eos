class_name HullData extends Sprite2D

@export var hull_script: Script

@export_subgroup("Defence")
@export var hull := 100
@export var armor := 100

@export_subgroup("Physic")
## Only support circle, rectangle and convex/concave polygon.
## Polygon will be decomposed into convex shapes.
##
## Tip:  Create a temporary CollisionPolygon2D node
## to help build polygon and copy/paste the points here.
@export var shape: Shape2D
@export var density := 1.0


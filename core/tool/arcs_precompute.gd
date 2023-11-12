@tool
extends Node2D


const STEP := 0.05

## An array of array of shape.
@export var shapes : Array[Array] = []
@export_range(-3.05, -STEP, STEP)
var start := -3.0

@export var compute := false:
	set(_value):
		shapes = []
		var circle := CircleShape2D.new()
		circle.radius = 1.0
		shapes.push_back([circle])
		
		var polygon : CollisionPolygon2D = $Area2D/CollisionPolygon2D
		
		var points := PackedVector2Array()
		var offset := start
		while offset < -STEP + 0.01:
			var circon := absf(offset + offset)
			var num_points := maxi(int(circon / 0.3), 3)
			if num_points % 2 == 0:
				num_points += 1
			
			var point_step := circon / float(num_points - 1)
			points.resize(num_points + 1)
			points[0] = Vector2.ZERO
			
			for i in num_points:
				var a := float(i) * point_step + offset
				points[i + 1] = Vector2(
					cos(a - PI * 0.5),
					sin(a - PI * 0.5))
			
			polygon.set_polygon(points)
			break
			start += STEP
		
			var area : Area2D = polygon.get_parent()
			var owners := area.get_shape_owners()
			if owners.size() != 1:
				push_error("this should be 1")
				if owners[0] != 0:
					push_error("this should be 0")
			var shape_count := area.shape_owner_get_shape_count(0)
			print(shape_count)
			for shape_id in shape_count:
				var shape : ConvexPolygonShape2D = area.shape_owner_get_shape(
					0, shape_id)
				print(shape)
		
		
		shapes.reverse()



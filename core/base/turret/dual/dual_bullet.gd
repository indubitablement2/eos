extends Bullet


var damage := 50.0


func _process(delta: float) -> void:
	super._process(delta)
	rotation = velocity.angle() + PI * 0.5


func _on_body_entered(hull: Hull) -> void:
	hull.damage(damage, position, Vector4.ONE)
	queue_free()

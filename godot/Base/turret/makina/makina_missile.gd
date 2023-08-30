extends Entity


func _ready() -> void:
	super._ready()
	
	await get_tree().create_timer(0.7, false, true).timeout
	wish_linvel_relative(Vector2(0.0, -1.0))
#	remove_collision_exception_with()
	
	await get_tree().create_timer(1.2, false, true).timeout
	body_entered.disconnect(_on_body_entered)
	modulate = Color.GRAY
	
	await get_tree().create_timer(0.4, false, true).timeout
	queue_free()


func _on_body_entered(body: Node) -> void:
	var e := body as Entity
	if e:
		e.damage(5000.0, position, 1.0)
		
	
	queue_free()

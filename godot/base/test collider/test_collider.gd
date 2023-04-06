extends GridCharacterBody

const ACCELERATION = Vector2(9.0, 0.0)
const GRAVITY = 10.0

func _physics_process(delta: float) -> void:
	var dir := Vector2(
		Input.get_action_strength("right") - Input.get_action_strength("left"),
		Input.get_action_strength("down") - Input.get_action_strength("up")
	)
	
	if Input.is_action_just_pressed("up"):
		velocity.y -= 5.0
	
	velocity += dir * ACCELERATION * delta
	velocity.y += GRAVITY * delta
	velocity *= 0.96
	move()
	
	var v := (velocity * 10.0).round() * 0.1
	$velocity.set_text(str(v))

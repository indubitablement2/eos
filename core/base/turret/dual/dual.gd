extends Turret


const BULLET := preload("res://base/turret/dual/dual_bullet.tscn")
const VELOCITY := -500.0
const DAMAGE := 50.0

var x := 4.0
const Y := -6.0


func fire() -> void:
	var b : Bullet = BULLET.instantiate()
	b.set_team(hull.team)
	
	b.ttl *= hull.projectile_range
	b.damage = DAMAGE * hull.projectile_damage
	b.velocity = Vector2(0.0, VELOCITY * hull.projectile_speed).rotated(global_rotation)
	
	position = Vector2(x, Y)
	b.position = global_position
	position = Vector2.ZERO
	x *= -1.0
	
	Battlescape.add_child(b)

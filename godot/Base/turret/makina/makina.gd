extends Turret


const missile := preload("res://Base/turret/makina/makina_missile.tscn")


@onready var missile1 := $MakinaMissile1
@onready var missile2 := $MakinaMissile2


func fire() -> void:
	var m : Entity = missile.instantiate()
	m.add_collision_exception_with(entity)
	m.global_transform = get_global_transform()
	Battlescape.add_child(m)

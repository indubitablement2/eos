extends Turret


const missile := preload("res://Base/turret/makina/makina_missile.tscn")


@onready var missile1 := $MakinaMissile1
@onready var missile2 := $MakinaMissile2


func fire() -> void:
	var m := missile.instantiate()
	add_child(m)

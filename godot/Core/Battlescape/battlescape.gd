extends Node2D


func _ready() -> void:
	Player.controlled = $Entity


func _process(_delta: float) -> void:
	$Label.text = str(
		$Entity.angular_velocity,
#		"\n",
#		$Entity.wish_angvel,
		"\n",
		$Entity.linear_velocity,
#		"\n",
#		$Entity.wish_linvel)
	)

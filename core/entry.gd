extends Node

func _ready() -> void:
	# Add default Battlescape so that playing scene from the editor
	# which rely on there being one present work.
	get_parent().add_child.call_deferred(Battlescape.new())

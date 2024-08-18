extends Node

func _ready() -> void:
	Battlescape._static_entry()
	
	# Add default Battlescape so that playing scene from the editor,
	# which rely on there being one present, work.
	get_parent().add_child.call_deferred(Battlescape.new())

func _enter_tree() -> void:
	Battlescape._static_exit()

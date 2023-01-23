extends Node2D

func _ready() -> void:
	$Client.try_load_data("res://entity test/ship_test.tscn")
	$Client.clear_data()
	
#	var s := preload("res://test_export.gd")
#	var i := s.new()
#	print(s)
#	i.inc(123)
#	var b = i.serde()
#	print(b)
#	var i2 = s.new()
#	i2.deserde(b)
#	i2.pr()
#	i2.inc(100)
#	i2.pr()
	
#	var asd = HullScript.new()
#	asd.__is_hull_script()
#	return

#	var n = preload("res://entity test/ship_test.tscn").instantiate()
#	var a := preload("res://entity test/entity_test.tscn").instantiate()
#	var b :EntityScript= a.simulation_script.new()
#	print(b.has_method("__is_entity_script"))
#	var r = preload("res://entity test/hull_sim.gd").new()
#	r.render_call(&"a", [])
#	print(r.has_method("__is_hull_script"))
	
#	var b: Node = load(n.entity_path).instantiate()
#	var s :Resource = b.get_child(0).simulation_script
#	var h = s._step()
#	print(h)
#	return
	
#	print(n.get_class())
#	print(n.has_method("_is_ship_data"))
#	$Client.try_load_data("res://entity test/ship_test.tscn")

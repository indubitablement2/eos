extends CanvasLayer

const SHIP_SCENE := preload("res://ui/ship.tscn")

@onready var grid :GridContainer = $TextureRect/VBoxContainer/HBoxContainer/ScrollContainer/GridContainer
@onready var total_cost_label :Label = $TextureRect/VBoxContainer/Label
@onready var accept_button :Button = $TextureRect/VBoxContainer/HBoxContainer/Accept

@onready var bs :ClientBattlescape = get_parent()
var max_active_cost := 0 : set = set_max_active_cost

var ship_cost :Array[int] = []
var active_cost := 0
var _last_toggle := true

#func _ready() -> void:
#	for i in 100:
#		await get_tree().create_timer(0.5).timeout
#		add_ship(preload("res://textures/spaceship gen.png"), randf_range(0.5, 1.0), str(i), i, randi() % 4 == 0)

func add_ship(icon: Texture2D, size_factor: float, tooptip: String, cost: int, destroyed: bool) -> void:
	var ship = SHIP_SCENE.instantiate()
	grid.add_child(ship)
	var idx := ship_cost.size()
	ship.toggled.connect(_on_ship_toggled.bind(ship, idx))
	ship.mouse_entered.connect(_on_ship_hovered.bind(ship))
	ship.set_ship(icon, size_factor, tooptip, cost, destroyed)
	ship_cost.push_back(cost)

func ship_set_ready(_idx: int) -> void:
	# Not implemented. Should never need to go back to ready.
	push_error("ship state going back to ready not implemented")

func ship_set_spawned(idx: int) -> void:
	# TODO: handle this by disabling button and showing some 'spawned' marker
	ship_set_destroyed(idx)

func ship_set_removed(idx: int) -> void:
	# TODO: Display counter when this ship can be spawned again.
	ship_set_destroyed(idx)

func ship_set_destroyed(idx: int) -> void:
	grid.get_child(idx).set_destroyed()

func set_max_active_cost(value: int) -> void:
	max_active_cost = value
	_update_cost()

func reset() -> void:
	active_cost = 0
	for ship in grid.get_children():
		ship.set_pressed_no_signal(false)

func _update_cost() -> void:
	total_cost_label.set_text(str(active_cost) + "/" + str(max_active_cost))
	if active_cost > max_active_cost:
		total_cost_label.set_modulate(Color(1.0, 0.22, 0.22))
		accept_button.set_disabled(true)
	else:
		total_cost_label.set_modulate(Color.ALICE_BLUE)
		accept_button.set_disabled(false)

func _on_ship_toggled(toggle: bool, ship: BaseButton, idx: int) -> void:
	if ship.is_disabled():
		return
	
	var cost := ship_cost[idx]
	if toggle:
		active_cost += cost
	else:
		active_cost -= cost
	_update_cost()
	
	_last_toggle = toggle

func _on_ship_hovered(ship: BaseButton) -> void:
	if Input.is_mouse_button_pressed(MOUSE_BUTTON_LEFT):
		ship.set_pressed(_last_toggle)

func _on_accept_pressed() -> void:
	if bs:
		var selected = PackedInt64Array([])
		var idx := 0
		for ship in grid.get_children():
			if ship.is_pressed():
				selected.push_back(idx)
			idx += 1
		bs.fleet_ship_selected(selected)
		print(selected)
	else:
		push_error("Can not select ships as not bind")

func _on_cancel_pressed() -> void:
	hide()
	reset()

func _on_reset_pressed() -> void:
	reset()

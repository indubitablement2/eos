extends ShipSelection

const SHIP_SCENE := preload("res://ui/ship.tscn")

@onready var grid :GridContainer = $TextureRect/VBoxContainer/HBoxContainer/ScrollContainer/GridContainer
@onready var total_cost_label :Label = $TextureRect/VBoxContainer/Label
@onready var accept_button :Button = $TextureRect/VBoxContainer/HBoxContainer/Accept

var max_active_cost := 0 : set = i_set_max_active_cost
var num_ship := 0
var active_cost := 0
var _last_toggle := true

#func _ready() -> void:
#	for i in 100:
#		await get_tree().create_timer(0.5).timeout
#		add_ship(preload("res://textures/spaceship gen.png"), randf_range(0.5, 1.0), str(i), i, randi() % 4 == 0)

func i_add_ship(icon: Texture2D, size_factor: float, tooptip: String, cost: int) -> int:
	var ship = SHIP_SCENE.instantiate()
	grid.add_child(ship)
	ship.set_ship(icon, size_factor, tooptip, cost)
	
	var idx := num_ship
	num_ship += 1
	
	ship.toggled.connect(_on_ship_toggled.bind(ship, idx, cost))
	ship.mouse_entered.connect(_on_ship_hovered.bind(ship))
	
	return idx

func i_ship_set_ready(_idx: int) -> void:
	pass

func i_ship_set_spawned(idx: int) -> void:
	# TODO: handle this by disabling button and showing some 'spawned' marker
	i_ship_set_destroyed(idx)

func i_ship_set_removed(idx: int) -> void:
	# TODO: Display counter when this ship can be spawned again.
	i_ship_set_destroyed(idx)

func i_ship_set_destroyed(idx: int) -> void:
	grid.get_child(idx).set_destroyed()

func i_set_max_active_cost(value: int) -> void:
	max_active_cost = value
	_update_cost()

func reset() -> void:
	for ship in grid.get_children():
		ship.set_pressed_no_signal(false)
	active_cost = 0
	clear_selection()

func _update_cost() -> void:
	total_cost_label.set_text(str(active_cost) + "/" + str(max_active_cost))
	
	if active_cost > max_active_cost:
		total_cost_label.set_modulate(Color(1.0, 0.22, 0.22))
		accept_button.set_disabled(true)
	else:
		total_cost_label.set_modulate(Color.ALICE_BLUE)
		accept_button.set_disabled(false)

func _on_ship_toggled(toggle: bool, ship: BaseButton, idx: int,  cost: int) -> void:
	if ship.is_disabled():
		return
	
	if toggle:
		active_cost += cost
		select(idx)
	else:
		active_cost -= cost
		deselect(idx)
	
	_update_cost()
	
	_last_toggle = toggle

func _on_ship_hovered(ship: BaseButton) -> void:
	if Input.is_mouse_button_pressed(MOUSE_BUTTON_LEFT):
		ship.set_pressed(_last_toggle)

func _on_accept_pressed() -> void:
	hide()
	spawn_selected()
	reset()

func _on_cancel_pressed() -> void:
	hide()
	reset()

func _on_reset_pressed() -> void:
	reset()

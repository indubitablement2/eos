@tool
extends RigidBody2D
class_name Entity2

const ARMOR_CELL_EFFECT := [
	[Vector2i(-1, -2), 0.5],
	[Vector2i(0, -2), 0.6],
	[Vector2i(1, -2), 0.5],
	
	[Vector2i(-2, -1), 0.5],
	[Vector2i(-1, -1), 1],
	[Vector2i(0, -1), 1],
	[Vector2i(1, -1), 1],
	[Vector2i(2, -1), 0.5],
	
	[Vector2i(-2, 0), 0.6],
	[Vector2i(-1, 0), 1],
	[Vector2i(0, 0), 1],
	[Vector2i(1, 0), 1],
	[Vector2i(2, 0), 0.6],
	
	[Vector2i(-2, 1), 0.5],
	[Vector2i(-1, 1), 1],
	[Vector2i(0, 1), 1],
	[Vector2i(1, 1), 1],
	[Vector2i(2, 1), 0.5],
	
	[Vector2i(-1, 2), 0.5],
	[Vector2i(0, 2), 0.6],
	[Vector2i(1, 2), 0.5],
]
const ARMOR_CELL_EFFECT_TOTAL := 15.4
const ARMOR_SCALE := 8.0
# TODO: Experiment with size of 3x3
const ARMOR_SIZE_MIN = Vector2i(5, 5);
const ARMOR_CENTER_MIN = Vector2i(2, 2);
const ARMOR_CENTER_MAX = Vector2i(3, 3);

const HULL_SHADER_PATH := "res://Core/shader/hull.gdshader"


enum EntityCollisionType {
	SHIP = 1 | 2 | 3,
	BANANA = 1 | 5 | 3,
}

enum ToolType {
	NOTHING,
	VERIFY,
	FETCH_TURRET_SLOTS,
}
@export var tool := ToolType.NOTHING : set = set_tool


@export var sprite : Texture2D : set = set_sprite
@export var sprite_offset := Vector2.ZERO : set = set_sprite_offset


@export var turret_slots : Array[TurretSlot] = []
@export var entity_collision_type := EntityCollisionType.SHIP


var turrets_primary : Array[Turret] = []
var turrets_secondary : Array[Turret] = []
var turrets_auto : Array[Turret] = []


var player_controlled := false
## INF used as a flag for turrets to take their default rotation.
var aim_at := Vector2.INF
## 0: none
## 1..14: just pressed actions (respective auto only flags also on)
## 14..28 auto only actions
var actions := 0


@export var hull_hp_max := 1000.0
@onready var hull_hp := hull_hp_max

@export var armor_max := 100.0

## Removes: recent damage, forced hull shader, armor texture.
## Used for projectiles.
@export var simple_armor := true : set = set_simple_armor

@export_group("Hidden")
## Maximum armor for each armor cell.
## Computed automaticaly.
## Should not be modified. Shared between instances.
@export var armor_max_relative : Image
@export_group("")
## Should not be modified at run time. Shared between instances.
@export var armor_max_relative_texture : Texture2D :
	set = set_armor_max_relative_texture


## How much armor for each cell.
## 1.0 == armor_max * ARMOR_CELL_EFFECT_TOTAL
var armor_relative : Image
## null if simple_armor
var armor_relative_texture : ImageTexture = null

var just_took_damage := false
var has_recent_damage := false
## null if simple_armor
var recent_damage : Image = null
## null if simple_armor
var recent_damage_texture : ImageTexture = null


func _ready() -> void:
	if Engine.is_editor_hint():
		_verify()
		return
	
	armor_relative = armor_max_relative.duplicate()
	if !simple_armor:
		armor_relative_texture = ImageTexture.create_from_image(armor_relative)
	
		recent_damage = Image.create(
			armor_relative.get_width(),
			armor_relative.get_height(),
			false,
			Image.FORMAT_RF)
		recent_damage_texture = ImageTexture.create_from_image(recent_damage)
	
		material.set_shader_parameter(
			&"armor_max_texture", armor_max_relative_texture)
		material.set_shader_parameter(&"armor_texture", armor_relative_texture)
		material.set_shader_parameter(
			&"recent_damage_texture", recent_damage_texture)


func _process(delta: float) -> void:
	if simple_armor:
		return
	
	if just_took_damage:
		armor_relative_texture.update(armor_relative)
		has_recent_damage = true
		just_took_damage = false
	
	if has_recent_damage:
		has_recent_damage = false
		
		var sub := delta * 0.25
		for y in recent_damage.get_height():
			for x in recent_damage.get_width():
				var v := recent_damage.get_pixel(x, y).r
				if v > sub:
					v -= sub
					has_recent_damage = true
				else:
					v = 0.0
				recent_damage.set_pixel(x, y, Color(v, 0.0, 0.0, 0.0))
		
		recent_damage_texture.update(recent_damage)
		
		queue_redraw()


func _integrate_forces(state: PhysicsDirectBodyState2D) -> void:
	# Contacts
	var contact_count := state.get_contact_count()
	var contact_idx := 0
	while contact_idx < contact_count:
		var contact_impulse := state.get_contact_impulse(contact_idx)
		
		var contact_lenght := contact_impulse.length_squared()
		if contact_lenght > 400.0:
			contact_lenght = sqrt(contact_lenght)
			var contact_position := state.get_contact_local_position(
			contact_idx)
			
			damage(contact_lenght, contact_position, 1.0)
			
			print(contact_lenght)
			print(contact_position)
		
		contact_idx += 1
	


func _draw() -> void:
	draw_texture(sprite, -sprite.get_size() * 0.5 + sprite_offset)


func damage(dmg: float, pos: Vector2, armor_dmg_multiplier : float) -> void:
	const min_armor := 0.1
	
	# Find nearest pixel.
	pos = to_local(pos)
	pos /= ARMOR_SCALE
	pos -= Vector2(0.5, 0.5)
	pos = pos.round()
	var pixel = Vector2i(pos)
	pixel += armor_relative.get_size() / 2
	pixel = pixel.clamp(
		Vector2i(2, 2), armor_relative.get_size() - Vector2i(3, 3))
	
	var a := 0.0
	for v in ARMOR_CELL_EFFECT:
		a += armor_relative.get_pixelv(pixel + v[0]).r * v[1]
	a /= ARMOR_CELL_EFFECT_TOTAL
	a = maxf(a, min_armor)
	
	var dmg_reduction := dmg / (dmg + a * armor_max)
	var hull_dmg := dmg * dmg_reduction
	var armor_dmg := (
		(hull_dmg / armor_max) / ARMOR_CELL_EFFECT_TOTAL) * armor_dmg_multiplier
	
	for v in ARMOR_CELL_EFFECT:
		a = armor_relative.get_pixelv(pixel + v[0]).r * v[1] - armor_dmg * v[1]
		armor_relative.set_pixelv(pixel + v[0], Color(a, 1.0, 1.0))
	
	if !simple_armor:
		var recent_damage_add := hull_dmg / (hull_hp_max * 0.1)
		for v in ARMOR_CELL_EFFECT:
			a = recent_damage.get_pixelv(pixel + v[0]).r
			a += recent_damage_add * v[1]
			a = minf(a, 1.0)
			recent_damage.set_pixelv(pixel + v[0], Color(a, 1.0, 1.0))
		
		just_took_damage = true
	
	hull_hp -= hull_dmg


func set_tool(value: ToolType) -> void:
	tool = ToolType.NOTHING
	match value:
		ToolType.NOTHING:
			pass
		ToolType.VERIFY:
			_verify()
		ToolType.FETCH_TURRET_SLOTS:
			turret_slots = []
			for c in get_children():
				if c is TurretSlot:
					turret_slots.push_back(c)


func set_simple_armor(value: bool) -> void:
	simple_armor = value
	_verify()

func set_sprite(value: Texture2D) -> void:
	sprite = value
	queue_redraw()
	_verify()

func set_sprite_offset(value: Vector2) -> void:
	sprite_offset = value
	queue_redraw()

func set_armor_max_relative_texture(value: Texture2D) -> void:
	armor_max_relative_texture = value
	_verify()


func _verify() -> void:
	if !Engine.is_editor_hint():
		return
	
	custom_integrator = true
	max_contacts_reported = 4
	contact_monitor = true
	can_sleep = false
	
	if !armor_max_relative_texture:
		armor_max_relative_texture = preload(
			"res://Core/texture/pixel.png")
	
	armor_max_relative = armor_max_relative_texture.get_image()
	armor_max_relative.convert(Image.FORMAT_R8)
	var s := Vector2i((sprite.get_size() / ARMOR_SCALE).ceil())
	armor_max_relative.resize(
		s.x,
		s.y,
		Image.INTERPOLATE_BILINEAR)
	
	if simple_armor:
		if material is ShaderMaterial:
			if material.shader.resource_path == HULL_SHADER_PATH:
				material = null
	else:
		material = ShaderMaterial.new()
		material.set_shader(preload(HULL_SHADER_PATH))
		
		material.resource_local_to_scene = true





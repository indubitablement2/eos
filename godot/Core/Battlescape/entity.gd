extends EntityBase
class_name Entity

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


enum EntityCollisionType {
	SHIP = 1 | 2 | 3,
	BANANA = 1 | 5 | 3,
}


@export var sprite : Texture2D
@export var sprite_offset := Vector2.ZERO


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


@export var hull_hp := 1000.0
@export var armor_max := 100.0

## Maximum armor for each armor cell.
## Should not be modified at run time. Shared between instances.
@export var armor_max_relative : Image
## Should not be modified at run time. Shared between instances.
@export var armor_max_relative_texture : ImageTexture

## How much armor for each cell.
## 1.0 == armor_max * ARMOR_CELL_EFFECT_TOTAL
var armor_relative : Image
var armor_relative_texture : ImageTexture

var just_took_damage := false
var has_recent_damage := false
var recent_damage : Image
var recent_damage_texture : ImageTexture


func _ready() -> void:
	var armor_textures_size := Vector2i(sprite.get_size() / ARMOR_SCALE)
	
	# TODO: Remove this
	var armor_relative_max_data := PackedByteArray()
	armor_relative_max_data.resize(
		armor_textures_size.x * armor_textures_size.y)
	armor_relative_max_data.fill(255)
	armor_max_relative = Image.create_from_data(
		armor_textures_size.x,
		armor_textures_size.y,
		false,
		Image.FORMAT_R8,
		armor_relative_max_data)
	armor_max_relative_texture = ImageTexture.create_from_image(
		armor_max_relative)
	# TODO: Remove this
	
	armor_relative = armor_max_relative.duplicate()
	armor_relative_texture = ImageTexture.create_from_image(armor_relative)
	
	recent_damage = Image.create(
		armor_textures_size.x,
		armor_textures_size.y,
		false,
		Image.FORMAT_R8)
	recent_damage_texture = ImageTexture.create_from_image(recent_damage)
	
	material.set_shader_parameter(
		&"armor_max_texture", armor_max_relative_texture)
	material.set_shader_parameter(&"armor_texture", armor_relative_texture)
	material.set_shader_parameter(
		&"recent_damage_texture", recent_damage_texture)
	
	
#	print("--------")
#	var r := []
#	var sum := 0.0
#	for y in range(-2, 3):
#		for x in range(-2, 3):
#			var vec := Vector2i(x, y)
#			var dist := Vector2(vec).length()
#			var eff := minf(2.82842707633972 - dist, 1.0)
#			if eff > 0.0:
#				r.push_back([vec, eff])
#				print("[Vector2i", vec, ", ", eff, "],")
#				sum += eff
#	print(sum)
#	sum = 0.0
#	for i in ARMOR_CELL_EFFECT:
#		sum += i[1]
#	print(sum)


func _process(delta: float) -> void:
	if just_took_damage:
		armor_relative_texture.update(armor_relative)
		has_recent_damage = true
		just_took_damage = false
	
	if has_recent_damage:
		has_recent_damage = false
		
		var recent_damage_data := recent_damage.get_data()
		var sub := maxi(int((delta * 0.25) * 255.0), 1)
		for i in recent_damage_data.size():
			if recent_damage_data[i] > sub:
				recent_damage_data[i] = recent_damage_data[i] - sub
				has_recent_damage = true
			else:
				recent_damage_data[i] = 0
		
		recent_damage.set_data(
			recent_damage.get_width(),
			recent_damage.get_height(),
			false,
			Image.FORMAT_R8,
			recent_damage_data)
		
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
			
			take_dmg(contact_lenght, to_local(contact_position))
			
			print(contact_lenght)
			print(contact_position)
		
		contact_idx += 1
	
	_base_integrate_forces(state)


func _draw() -> void:
	draw_texture(sprite, -sprite.get_size() * 0.5 + sprite_offset)
#	draw_set_transform(Vector2.ZERO, 0.0, Vector2(ARMOR_SCALE, ARMOR_SCALE))
#	draw_texture(
#		recent_damage_texture, -armor_relative_texture.get_size() * 0.5)


func take_dmg(amount: float, location: Vector2) -> void:
	const min_armor := 0.1
	
	# Find nearest pixel.
	location /= ARMOR_SCALE
	location -= Vector2(0.5, 0.5)
	location = location.round()
	
	var pixel := Vector2i(location)
	pixel += armor_relative.get_size() / 2
	pixel = pixel.clamp(
		Vector2i(2, 2), armor_relative.get_size() - Vector2i(3, 3))
	
	var a := 0.0
	for v in ARMOR_CELL_EFFECT:
		a += armor_relative.get_pixelv(pixel + v[0]).r * v[1]
	a /= ARMOR_CELL_EFFECT_TOTAL
	a = maxf(a, min_armor)
	
	var dmg_reduction := amount / (amount + a * armor_max)
	var dmg := amount * dmg_reduction
	var armor_dmg := (dmg / armor_max) / ARMOR_CELL_EFFECT_TOTAL
	
	for v in ARMOR_CELL_EFFECT:
		a = armor_relative.get_pixelv(pixel + v[0]).r * v[1] - armor_dmg * v[1]
		armor_relative.set_pixelv(pixel + v[0], Color(a, 1.0, 1.0))
		
		a = recent_damage.get_pixelv(pixel + v[0]).r + armor_dmg * v[1]
		recent_damage.set_pixelv(pixel + v[0], Color(a, 1.0, 1.0))
	
	hull_hp -= dmg
	
	just_took_damage = true







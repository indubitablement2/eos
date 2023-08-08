extends EntityBase
class_name Entity


const ARMOR_CELL_FULL : Array[Vector2i] = [
	Vector2i(-1, -1),
	Vector2i(0, -1),
	Vector2i(1, -1),
	
	Vector2i(-1, 0),
	Vector2i(0, 0),
	Vector2i(1, 0),
	
	Vector2i(-1, 1),
	Vector2i(0, 1),
	Vector2i(1, 1),
]
const ARMOR_CELL_HALF : Array[Vector2i] = [
	Vector2i(-1, -2),
	Vector2i(0, -2),
	Vector2i(1, -2),
	
	Vector2i(-2, -1),
	Vector2i(-2, 0),
	Vector2i(-2, 1),
	
	Vector2i(2, -1),
	Vector2i(2, 0),
	Vector2i(2, 1),

	Vector2i(-1, 2),
	Vector2i(0, 2),
	Vector2i(1, 2),
]

const ARMOR_SCALE := 8.0


enum EntityCollisionType {
	SHIP = 1 | 2 | 3,
	BANANA = 1 | 5 | 3,
}


@export var hull_srpite : HullSprite
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


var hull_hp := 1000.0
var max_armor := 100.0

var armor : Image
var armor_texture : ImageTexture
var just_took_damage := false
var has_recent_damage := false
var recent_damage : Image
var recent_damage_texture : ImageTexture


func _ready() -> void:
	var armor_data := PackedByteArray()
	armor_data.resize(32*32)
	armor_data.fill(255)
	armor = Image.create_from_data(32, 32, false,Image.FORMAT_R8, armor_data)
	armor_texture = ImageTexture.create_from_image(armor)
	
	recent_damage = Image.create(32, 32, false, Image.FORMAT_R8)
	recent_damage_texture = ImageTexture.create_from_image(recent_damage)


func _process(delta: float) -> void:
	if just_took_damage:
		armor_texture.update(armor)
		has_recent_damage = true
		just_took_damage = false
	
	if has_recent_damage:
		has_recent_damage = false
		
		var recent_damage_data := recent_damage.get_data()
		for p in recent_damage_data:
			p = int(maxf(float(p) - delta, 0.0))
			has_recent_damage = has_recent_damage || p > 0
		
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
	draw_set_transform(Vector2.ZERO, 0.0, Vector2(ARMOR_SCALE, ARMOR_SCALE))
	draw_texture(armor_texture, - armor_texture.get_size() * 0.5)


func take_dmg(amount: float, location: Vector2) -> void:
	const min_armor := 0.1
	
	# Find nearest pixel.
	location /= ARMOR_SCALE
	location -= Vector2(0.5, 0.5)
	location = location.round()
	
	var pixel := Vector2i(location)
	pixel += armor.get_size() / 2
	pixel = pixel.clamp(Vector2i(2, 2), armor.get_size() - Vector2i(3, 3))
	
	var a := 0.0
	for offset in ARMOR_CELL_FULL:
		a += armor.get_pixelv(pixel + offset).r
	for offset in ARMOR_CELL_HALF:
		a += armor.get_pixelv(pixel + offset).r * 0.5
	a *= 0.066666667
	a = maxf(a, min_armor)
	
	var dmg_reduction := amount / (amount + a * max_armor)
	var dmg := amount * dmg_reduction
	var armor_dmg := (dmg / max_armor) * 0.066666667
	
	for offset in ARMOR_CELL_FULL:
		a = maxf(armor.get_pixelv(pixel + offset).r - armor_dmg, 0.0)
		armor.set_pixelv(pixel + offset, Color(a, 1.0, 1.0))
		
		a = recent_damage.get_pixelv(pixel + offset).r + armor_dmg
		recent_damage.set_pixelv(pixel + offset, Color(a, 1.0, 1.0))
	armor_dmg *= 0.5
	for offset in ARMOR_CELL_HALF:
		a = maxf(armor.get_pixelv(pixel + offset).r - armor_dmg, 0.0)
		armor.set_pixelv(pixel + offset, Color(a, 1.0, 1.0))
		
		a = recent_damage.get_pixelv(pixel + offset).r + armor_dmg
		recent_damage.set_pixelv(pixel + offset, Color(a, 1.0, 1.0))
	
	hull_hp -= dmg
	
	just_took_damage = true



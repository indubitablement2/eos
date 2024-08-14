extends Object
class_name Global

const COLLISION_FRIEND_SHIP := 1
const COLLISION_ENEMY_SHIP := 16
const COLLISION_FRIEND_FIGHTER := 2
const COLLISION_ENEMY_FIGHTER := 32
const COLLISION_FRIEND_MISSILE := 4
const COLLISION_ENEMY_MISSILE := 64
const COLLISION_FRIEND_PROJECTILE := 8
const COLLISION_ENEMY_PROJECTILE := 128
const COLLISION_DEBRIS := 268435456

static var _collision_mask_cache := {}
## Return value is cached, so you can call this as much as you need.
static func make_collision_mask(team: int, mask: int) -> int:
	var key := team | (mask << 8)
	var value = _collision_mask_cache.find_key(key)
	if value:
		return value
	
	team *= 4
	var ret := 0
	ret |= mask & COLLISION_FRIEND_SHIP & (1 << team)
	ret |= mask & COLLISION_ENEMY_SHIP & 17895697 & ~(1 << team)
	ret |= mask & COLLISION_FRIEND_FIGHTER & (2 << team)
	ret |= mask & COLLISION_ENEMY_FIGHTER & 35791394 & ~(2 << team)
	ret |= mask & COLLISION_FRIEND_MISSILE & (4 << team)
	ret |= mask & COLLISION_ENEMY_MISSILE & 71582788 & ~(4 << team)
	ret |= mask & COLLISION_FRIEND_PROJECTILE & (8 << team)
	ret |= mask & COLLISION_ENEMY_PROJECTILE & 143165576 & ~(8 << team)
	ret |= mask & COLLISION_DEBRIS
	
	_collision_mask_cache[key] = ret
	
	return ret

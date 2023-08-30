extends Turret
class_name TurretDummyProjectile


## Needs at least one dummy.
var dummies : Array[DummyProjectile]
var despawned : Array[DummyProjectile]


func _enter_tree() -> void:
	super._enter_tree()
	dummies = []
	despawned = []


func _physics_process(delta: float) -> void:
	super._physics_process(delta)
	
	var i := 0
	while i < despawned.size():
		despawned[i].cooldown_remaining -= delta
		if despawned[i].cooldown_remaining < 0.0 && ammo > dummies.size():
			var dummy := swap_remove(despawned, i)
			dummies.push_back(dummy)
			dummy.reload()
		else:
			i += 1
	
	while ammo < dummies.size() && !dummies.is_empty():
		var dummy : DummyProjectile = dummies.pop_front()
		if dummy:
			dummy.remove()
			despawned.push_back(dummy)


func fire() -> void:
	var dummy : DummyProjectile = dummies.pop_front()
	if dummy == null:
		if despawned.is_empty():
			push_error(
				"TurretDummyProjectile needs at least one DummyProjectile")
			return
		dummy = swap_remove(despawned, _lowest_dummy_cooldown())

	dummy.fire(self)
	
	dummy.cooldown_remaining = dummy.cooldown
	despawned.push_back(dummy)
	
	fire_cooldown = maxf(fire_cooldown, _highest_dummy_cooldown())

func _lowest_dummy_cooldown() -> int:
	var i := 0
	var lowest := INF
	var lowest_idx := -1
	while i < despawned.size():
		if despawned[i].cooldown_remaining < lowest:
			lowest = despawned[i].cooldown_remaining
			lowest_idx = i
		i += 1
	
	return lowest_idx

func _highest_dummy_cooldown() -> float:
	var highest := -INF
	for dummy in despawned:
		if dummy.cooldown_remaining > highest:
			highest = dummy.cooldown_remaining
	
	return highest


func swap_remove(array :Array[DummyProjectile], idx: int) -> DummyProjectile:
	var d := array[idx]
	array.remove_at(idx)
	return d

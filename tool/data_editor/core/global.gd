extends Node

const RENDER_SCALE = 256.0
const SIMULATION_SCALE = 1.0 / RENDER_SCALE

func scale2sim(value: float) -> float:
	return value * SIMULATION_SCALE

func vec2arr(vec: Vector2, scale: bool = true) -> Array[float]:
	var x = vec.x
	var y = vec.y
	
	if scale:
		x = scale2sim(x)
		y = scale2sim(y)
	
	x = roundf(x * 10000000) / 10000000
	y = roundf(y * 10000000) / 10000000
	
	return [x, y]

func packedvec2arr(packed: PackedVector2Array, scale: bool = true) -> Array:
	return Array(packed).map(func(vec): return vec2arr(vec, scale))

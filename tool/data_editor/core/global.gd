extends Node

const RENDER_SCALE = 256.0
const SIMULATION_SCALE = 1.0 / RENDER_SCALE

func sim_scale(value: float) -> float:
	return value * SIMULATION_SCALE

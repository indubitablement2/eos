#ifndef GRID
#define GRID

#include "core/math/vector2i.h"
#include "preludes.h"
#include "rng.hpp"
#include "scene/2d/node_2d.h"
#include "terrain.h"

class Grid : public Node2D {
	GDCLASS(Grid, Node2D);

public:
	struct Tile {
		Ref<Terrain> terrain = fallback_terrain;
		// floor
		// funiture (plant, tree, wall)
		// filth
		// pawns
		// task (collect, pickup, build)
	};

	constexpr static const f32 TILE_SIZE = 64.0f;

private:
	inline static const Ref<Terrain> fallback_terrain = Ref<Terrain>();

protected:
	static void _bind_methods();

public:
	std::vector<Tile> grid = std::vector<Tile>();

	RandomPCG rng = RandomPCG();
	void set_seed(u64 seed) {
		rng = RandomPCG(seed);
	}
	f32 randf() {
		return rng.randf();
	}
	f32 rand_rangef(f32 min, f32 max) {
		return rng.random(min, max);
	}
	bool rand_probability(f32 probability) {
		return rng.randf() < probability;
	}
	i32 randi() {
		return rng.rand();
	}
	i32 rand_rangei(i32 min, i32 max) {
		return rng.random(min, max);
	}

	ADD_SETGET(Vector2i, size, Vector2i())
};

#endif
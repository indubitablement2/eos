#ifndef GRID_H
#define GRID_H

#include "core/math/vector2.h"
#include "core/object/object.h"
#include "enemy.h"
#include "preludes.h"
#include <vector>

class Grid : public Object {
	GDCLASS(Grid, Object);

public:
	struct Collider {
		Vector2 position;
		f32 radius;
		i32 enemy_idx;
	};

protected:
	static void _bind_methods();

public:
	inline static i32 width = 0;
	inline static i32 height = 0;
	inline static f32 cell_size = 0.0f;
	static void new_empty(f32 wish_width, f32 wish_height, f32 wish_cell_size);

	inline static std::vector<std::vector<Collider>> cells;
	inline static std::vector<Enemy *> enemies;

	static void add_enemy(Enemy *enemy);

	static TypedArray<Enemy> query(Vector2 position, f32 radius);
};

#endif
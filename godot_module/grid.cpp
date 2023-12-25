#include "grid.h"
#include "core/object/class_db.h"
#include "core/typedefs.h"
#include "enemy.h"
#include <unordered_set>

void Grid::_bind_methods() {
	ClassDB::bind_static_method("Grid", D_METHOD("new_empty", "width", "height", "cell_size"), &Grid::new_empty);
	ClassDB::bind_static_method("Grid", D_METHOD("query", "position", "radius"), &Grid::query);
}

void Grid::new_empty(f32 wish_width, f32 wish_height, f32 wish_cell_size) {
	cell_size = MAX(wish_cell_size, 1.0f);

	width = CLAMP(i32(wish_width / cell_size) + 1, 1, 2048);
	height = CLAMP(i32(wish_height / cell_size) + 1, 1, 2048);

	cells.resize(width * height);
	for (i32 i = 0; i < cells.size(); i++) {
		cells[i].clear();
	}

	enemies.clear();
}

void Grid::add_enemy(Enemy *enemy) {
	i32 enemy_idx = enemies.size();
	enemies.push_back(enemy);

	Collider collider = Collider{
		.position = enemy->get_position(),
		.radius = enemy->get_radius(),
		.enemy_idx = enemy_idx
	};

	i32 x_start = CLAMP(i32((collider.position.x - collider.radius) / cell_size), 0, width);
	i32 x_end = CLAMP(i32((collider.position.x + collider.radius) / cell_size) + 1, 0, width);
	i32 y_start = CLAMP(i32((collider.position.y - collider.radius) / cell_size), 0, height);
	i32 y_end = CLAMP(i32((collider.position.y + collider.radius) / cell_size) + 1, 0, height);

	for (i32 x = x_start; x < x_end; x++) {
		for (i32 y = y_start; y < y_end; y++) {
			cells[x + y * width].push_back(collider);
		}
	}
}

TypedArray<Enemy> Grid::query(Vector2 position, f32 radius) {
	i32 x_start = CLAMP(i32((position.x - radius) / cell_size), 0, width);
	i32 x_end = CLAMP(i32((position.x + radius) / cell_size) + 1, 0, width);
	i32 y_start = CLAMP(i32((position.y - radius) / cell_size), 0, height);
	i32 y_end = CLAMP(i32((position.y + radius) / cell_size) + 1, 0, height);

	std::unordered_set<i32> set = std::unordered_set<i32>();
	for (i32 x = x_start; x < x_end; x++) {
		for (i32 y = y_start; y < y_end; y++) {
			i32 i = x + y * width;

			for (i32 j = 0; j < cells[i].size(); j++) {
				Collider collider = cells[i][j];

				f32 threshold = collider.radius + radius;

				if (collider.position.distance_squared_to(position) < threshold * threshold) {
					if (enemies[collider.enemy_idx]->is_queued_for_deletion()) {
						continue;
					}

					set.insert(collider.enemy_idx);
				}
			}
		}
	}

	TypedArray<Enemy> arr = TypedArray<Enemy>();
	arr.resize(set.size());
	i32 i = 0;
	for (auto it = set.begin(); it != set.end(); ++it) {
		arr[i] = enemies[*it];
		i++;
	}

	return arr;
}
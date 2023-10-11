#include "grid.h"
#include "core/variant/variant.h"
#include "preludes.h"

void Grid::set_size(Vector2i value) {
	size = value;
	grid.resize(size.x * size.y);
}

Vector2i Grid::get_size() const {
	return size;
}

void Grid::_bind_methods() {
	BIND_CONSTANT(TILE_SIZE);

	ADD_SETGET_PROPERTY(Grid, Variant::VECTOR2I, size);

	ClassDB::bind_method(
			D_METHOD("set_seed", "seed"),
			&Grid::set_seed);
	ClassDB::bind_method(
			D_METHOD("randf"),
			&Grid::randf);
	ClassDB::bind_method(
			D_METHOD("rand_rangef", "min", "max"),
			&Grid::rand_rangef);
	ClassDB::bind_method(
			D_METHOD("rand_probability", "probability"),
			&Grid::rand_probability);
	ClassDB::bind_method(
			D_METHOD("randi"),
			&Grid::randi);
	ClassDB::bind_method(
			D_METHOD("rand_rangei", "min", "max"),
			&Grid::rand_rangei);
}

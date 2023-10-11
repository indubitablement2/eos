#ifndef PAWN
#define PAWN

#include "core/math/vector2i.h"
#include "grid.h"
#include "preludes.h"
#include "scene/2d/node_2d.h"

class Pawn : public Node2D {
	GDCLASS(Pawn, Node2D);

public:
	struct QueuedMovement {
		Callable callback;

		// Front is target, back is closest tile to us.
		std::vector<Vector2i> path;

		// How much time has been put into moving to the next tile.
		f32 movement_progress;
		// How much time it takes to move to the next tile.
		f32 movement_cost;
		f32 time_since_last_checked_path;

		Vector2i target;
		// If we only want to move to a tile next to the target tile.
		bool next_to;
		// If we have computed the path.
		bool is_ready;
	};

private:
	void finish_path(bool success);
	bool try_start_next_path();

protected:
	static void _bind_methods();

	void _notification(int what);

public:
	std::vector<QueuedMovement> queued_movement = std::vector<QueuedMovement>();
	void queue_movement(Vector2i coords, Callable callback, bool next_to = false);
	void clear_movement();
	bool is_moving() const { return !queued_movement.empty(); }

	Grid *get_grid() const { return Object::cast_to<Grid>(get_parent()); }

	ADD_SETGET(Vector2i, coordinates, Vector2i())
	// ADD_SETGET(Vector2i, coordinates, Vector2i())

	ADD_SETGET(f32, movement_speed, 1.0f)
};

#endif
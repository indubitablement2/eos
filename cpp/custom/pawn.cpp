#include "pawn.h"
#include "core/math/vector2i.h"
#include "core/string/print_string.h"
#include "core/variant/variant.h"
#include "grid.h"
#include "preludes.h"
#include <cmath>

ADD_SETGET_IMPL(Pawn, f32, movement_speed)

void Pawn::_bind_methods() {
	ADD_SETGET_PROPERTY(Pawn, Variant::VECTOR2I, coordinates)
	ADD_SETGET_PROPERTY(Pawn, Variant::FLOAT, movement_speed)

	ClassDB::bind_method(
			D_METHOD("queue_movement", "target", "callback", "next_to"),
			&Pawn::queue_movement);
	ClassDB::bind_method(
			D_METHOD("clear_movement"),
			&Pawn::clear_movement);
	ClassDB::bind_method(
			D_METHOD("is_moving"),
			&Pawn::is_moving);

	// ADD_SIGNAL(MethodInfo(
	// 		"movement_finished",
	// 		PropertyInfo(Variant::BOOL, "success")));
}

void Pawn::finish_path(bool success) {
	Callable cb = queued_movement[0].callback;
	queued_movement.erase(queued_movement.begin());

	if (cb.is_valid()) {
		const Variant arg = Variant(success);
		const Variant *arg_ptr = &arg;
		const Variant **arg_ptr_ptr = &arg_ptr;
		Variant ret;
		Callable::CallError err;
		cb.callp(
				arg_ptr_ptr,
				1,
				ret,
				err);
	}
}

// Returns true when next path is ready.
bool Pawn::try_start_next_path() {
	while (!queued_movement.empty()) {
		QueuedMovement &m = queued_movement.front();

		// Start a new path.
		if (!m.is_ready) {
			m.is_ready = true;

			// TODO: Compute path.
			Vector2i last = m.target;
			if (last == coordinates) {
				// Already on target.
				finish_path(true);
				continue;
			}
			// m.path.push_back(last);
			while (last != coordinates) {
				Vector2i dif = coordinates - last;

				dif = dif.clamp(
						Vector2i(-1, -1),
						Vector2i(1, 1));

				m.path.push_back(last);
				last = last + dif;
			}

			// TODO: Compute cost to move to first tile.
			m.movement_cost = 1.0f;
		}

		// Path is ready
		return true;
	}

	// No path
	return false;
}

void Pawn::_notification(int what) {
	if (what == NOTIFICATION_PROCESS) {
		f32 delta = (f32)get_process_delta_time();

		if (try_start_next_path()) {
			QueuedMovement &m = queued_movement.front();

			// TODO: Validate path.
			m.time_since_last_checked_path += delta;
			if (m.time_since_last_checked_path > 20.0f) {
				// Verify that all tile in path are still passable.
				m.time_since_last_checked_path = 0.0f;
			} else {
				// Verify that next tile in path is still passable.
			}

			m.movement_progress += delta;
			// print_line(m.movement_progress, "/", m.movement_cost);

			// Move along the path.
			while (m.movement_progress > m.movement_cost) {
				f32 over_progress = m.movement_progress - m.movement_cost;

				auto &path = m.path;

				set_coordinates(path.back());
				path.pop_back();

				if (path.empty()) {
					// Finished this path.
					finish_path(true);
					if (!try_start_next_path()) {
						break;
					}

					m = queued_movement.front();
				} else {
					// TODO: Compute cost to move to the next tile.
					// TODO: Check if moving next tile is blocked.
					m.movement_cost = 1.0f;
				}

				m.movement_progress = over_progress;
			}
		}
	}
}

void Pawn::queue_movement(Vector2i target, Callable callback, bool next_to) {
	// TODO: Check that target is within bounds.
	TEST_ASSERT(true, "Target is out of bounds.");

	QueuedMovement movement = QueuedMovement{
		callback,
		std::vector<Vector2i>(),
		0.0f,
		0.0f,
		0.0f,
		target,
		next_to,
		false
	};
	queued_movement.push_back(movement);
}

void Pawn::clear_movement() {
	while (!queued_movement.empty()) {
		finish_path(false);
	}
	// TODO: Reset position to current cell center.
}

void Pawn::set_coordinates(Vector2i coords) {
	print_line(coordinates, "->", coords);
	// TODO: clear paths.
	coordinates = coords;
	// TODO: Smooth position.
	set_position(Vector2((f32)coords.x, (f32)coords.y) * Grid::TILE_SIZE);
	// TODO: Check if there are any callback when a pawn enter this cell.
}

Vector2i Pawn::get_coordinates() const {
	return coordinates;
}
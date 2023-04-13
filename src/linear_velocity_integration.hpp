#ifndef LINEAR_VELOCITY_INTEGRATION_HPP
#define LINEAR_VELOCITY_INTEGRATION_HPP

#include "godot_cpp/godot.hpp"
#include "godot_cpp/variant/utility_functions.hpp"
#include "godot_cpp/variant/vector2.hpp"

using namespace godot;

namespace LinearVelocityIntegration {

// How much to increase acceleration force when stopping.
const float STOP_ACCELERATION_MULTIPLIER = 1.05f;

// Return the linear velocity after applying a force to stop.
inline Vector2 stop(Vector2 linear_velocity, float linear_acceleration, float delta) {
	return linear_velocity - linear_velocity.limit_length(linear_acceleration * STOP_ACCELERATION_MULTIPLIER * delta);
}

// Return the linear velocity after applying a force to reach wish_linear_velocity.
// Does not care about max velocity.
// wish_linear_velocity should already be capped.
inline Vector2 wish(Vector2 wish_linear_velocity, Vector2 linear_velocity, float linear_acceleration, float delta) {
	return linear_velocity + (wish_linear_velocity - linear_velocity).limit_length(linear_acceleration * delta);
}

} // namespace LinearVelocityIntegration

#endif

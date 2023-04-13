#ifndef ANGULAR_VELOCITY_INTEGRATION_HPP
#define ANGULAR_VELOCITY_INTEGRATION_HPP

#include "godot_cpp/godot.hpp"
#include "godot_cpp/variant/vector2.hpp"

using namespace godot;

namespace AngularVelocityIntegration {
// How much to increase acceleration force when stopping.
const float STOP_ACCELERATION_MULTIPLIER = 1.05f;

// Return the angular velocity after applying a force to stop.
inline float stop(float angular_velocity, float angular_acceleration, float delta) {
	float max_deceleration = angular_acceleration * STOP_ACCELERATION_MULTIPLIER * delta;
	return angular_velocity - Math::clamp(angular_velocity, -max_deceleration, max_deceleration);
}

// Return the angular velocity after applying a force.
// force is assumed to be in the range [-1, 1].
inline float force(float force, float angular_velocity, float angular_acceleration, float max_angular_velocity, float delta) {
	if (angular_velocity > max_angular_velocity) {
		if (Math::sign(force) == Math::sign(angular_velocity)) {
			// Trying to go in the same dir as current velocity while speed is over max.
			// Ignore force, slow down to max speed instead.
			return Math::max(angular_velocity - angular_acceleration * delta, max_angular_velocity);
		} else {
			// Trying to go in the opposite dir as current velocity while speed is over max.
			float maybe = angular_velocity + force * angular_acceleration * delta;
			if (maybe > max_angular_velocity) {
				// Ignore force, slow down as much as possible to reach max speed instead.
				return Math::max(angular_velocity - angular_acceleration * delta, max_angular_velocity);
			} else {
				// Force is enough to slow down to max speed.
				return Math::max(maybe, -max_angular_velocity);
			}
		}
	} else if (angular_velocity < -max_angular_velocity) {
		if (Math::sign(force) == Math::sign(angular_velocity)) {
			// Trying to go in the same dir as current velocity while speed is over max.
			// Ignore force, slow down to max speed instead.
			return Math::min(angular_velocity + angular_acceleration * delta, -max_angular_velocity);
		} else {
			// Trying to go in the opposite dir as current velocity while speed is over max.
			float maybe = angular_velocity + force * angular_acceleration * delta;
			if (maybe < -max_angular_velocity) {
				// Ignore force, slow down as much as possible to reach max speed instead.
				return Math::min(angular_velocity + angular_acceleration * delta, -max_angular_velocity);
			} else {
				// Force is enough to slow down to max speed.
				return Math::min(maybe, max_angular_velocity);
			}
		}
	} else {
		// Speed is under max.
		return Math::clamp(angular_velocity + force * angular_acceleration * delta, -max_angular_velocity, max_angular_velocity);
	}
}

/// Return the angular velocity after applying a force to rotate by offset radian.
inline float offset(float offset, float angular_velocity, float angular_acceleration, float max_angular_velocity, float delta) {
	if (Math::abs(offset) < 0.01f) {
		return stop(angular_velocity, angular_acceleration, delta);
	} else if (Math::sign(offset) == Math::sign(angular_velocity)) {
		// Calculate the time to reach 0 angular velocity.
		float time_to_stop = Math::abs(angular_velocity / angular_acceleration);

		// Calculate the time to reach the target.
		float time_to_target = Math::abs(offset / angular_velocity);

		if (time_to_target < time_to_stop) {
			// We will overshoot the target, so we need to slow down.
			return stop(angular_velocity, angular_acceleration, delta);
		} else {
			// We can go at full speed.
			return force(Math::sign(offset), angular_velocity, angular_acceleration, max_angular_velocity, delta);
		}
	} else {
		// We are going in the opposite direction, so we can go at full speed.
		return force(Math::sign(offset), angular_velocity, angular_acceleration, max_angular_velocity, delta);
	}
}
} // namespace AngularVelocityIntegration

#endif

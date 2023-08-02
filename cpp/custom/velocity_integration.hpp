#ifndef VELOCITY_INTEGRATION_HPP
#define VELOCITY_INTEGRATION_HPP

#include "preludes.h"

namespace VelocityIntegration {

// Return the linear velocity after applying a force to stop.
inline Vector2 stop_linvel(Vector2 linvel, f32 linacc, f32 delta) {
	return linvel - linvel.limit_length(linacc * delta);
}

// Return the linear velocity after applying a force to reach wish_linvel.
// Does not care about max velocity.
// wish_linvel should already be capped.
inline Vector2 linvel(Vector2 wish_linvel, Vector2 linvel, f32 linacc, f32 delta) {
	return linvel + (wish_linvel - linvel).limit_length(linacc * delta);
}

// Return the angular velocity after applying a force to stop.
inline f32 stop_angvel(f32 angvel, f32 angacc, f32 delta) {
	return angvel - CLAMP(angvel, -angacc * delta, angacc * delta);
}

// Return the angular velocity after applying a force to reach wish_angvel.
// Does not care about max velocity.
// wish_angvel should already be capped.
inline f32 angvel(f32 wish_angvel, f32 angvel, f32 angacc, f32 delta) {
	return angvel + CLAMP(wish_angvel - angvel, -angacc * delta, angacc * delta);
}

} //namespace VelocityIntegration

#endif
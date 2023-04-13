#include "eos_entity.h"
#include "angular_velocity_integration.hpp"
#include "godot_cpp/classes/physics_direct_body_state2d.hpp"
#include "godot_cpp/core/class_db.hpp"
#include "godot_cpp/core/math.hpp"
#include "godot_cpp/variant/utility_functions.hpp"
#include "godot_cpp/variant/vector2.hpp"
#include "linear_velocity_integration.hpp"

using namespace godot;

const float DELTA = 1.0f / 60.0f;

void EosEntity::_bind_methods() {
	// BIND_CONSTANT(DELTA);

	ClassDB::bind_method(D_METHOD("set_base_linear_acceleration", "value"), &EosEntity::set_base_linear_acceleration);
	ClassDB::bind_method(D_METHOD("get_base_linear_acceleration"), &EosEntity::get_base_linear_acceleration);
	ADD_PROPERTY(PropertyInfo(Variant::FLOAT, "base_linear_acceleration"), "set_base_linear_acceleration", "get_base_linear_acceleration");
	ClassDB::bind_method(D_METHOD("set_base_angular_acceleration", "value"), &EosEntity::set_base_angular_acceleration);
	ClassDB::bind_method(D_METHOD("get_base_angular_acceleration"), &EosEntity::get_base_angular_acceleration);
	ADD_PROPERTY(PropertyInfo(Variant::FLOAT, "base_angular_acceleration"), "set_base_angular_acceleration", "get_base_angular_acceleration");
	ClassDB::bind_method(D_METHOD("set_base_max_linear_velocity", "value"), &EosEntity::set_base_max_linear_velocity);
	ClassDB::bind_method(D_METHOD("get_base_max_linear_velocity"), &EosEntity::get_base_max_linear_velocity);
	ADD_PROPERTY(PropertyInfo(Variant::FLOAT, "base_max_linear_velocity"), "set_base_max_linear_velocity", "get_base_max_linear_velocity");
	ClassDB::bind_method(D_METHOD("set_base_max_angular_velocity", "value"), &EosEntity::set_base_max_angular_velocity);
	ClassDB::bind_method(D_METHOD("get_base_max_angular_velocity"), &EosEntity::get_base_max_angular_velocity);
	ADD_PROPERTY(PropertyInfo(Variant::FLOAT, "base_max_angular_velocity"), "set_base_max_angular_velocity", "get_base_max_angular_velocity");

	ClassDB::bind_method(D_METHOD("set_base_readiness", "value"), &EosEntity::set_base_readiness);
	ClassDB::bind_method(D_METHOD("get_base_readiness"), &EosEntity::get_base_readiness);
	ADD_PROPERTY(PropertyInfo(Variant::FLOAT, "base_readiness"), "set_base_readiness", "get_base_readiness");

	ClassDB::bind_method(D_METHOD("set_readiness", "value"), &EosEntity::set_readiness);
	ClassDB::bind_method(D_METHOD("get_readiness"), &EosEntity::get_readiness);
	ADD_PROPERTY(PropertyInfo(Variant::FLOAT, "readiness"), "set_readiness", "get_readiness");
	ClassDB::bind_method(D_METHOD("set_hull_hp", "value"), &EosEntity::set_hull_hp);
	ClassDB::bind_method(D_METHOD("get_hull_hp"), &EosEntity::get_hull_hp);
	ADD_PROPERTY(PropertyInfo(Variant::FLOAT, "hull_hp"), "set_hull_hp", "get_hull_hp");
	ClassDB::bind_method(D_METHOD("set_average_armor_hp", "value"), &EosEntity::set_average_armor_hp);
	ClassDB::bind_method(D_METHOD("get_average_armor_hp"), &EosEntity::get_average_armor_hp);
	ADD_PROPERTY(PropertyInfo(Variant::FLOAT, "average_armor_hp"), "set_average_armor_hp", "get_average_armor_hp");

	ClassDB::bind_method(D_METHOD("set_wish_linear_velocity_keep"), &EosEntity::set_wish_linear_velocity_keep);
	ClassDB::bind_method(D_METHOD("set_wish_linear_velocity_cancel"), &EosEntity::set_wish_linear_velocity_cancel);
	ClassDB::bind_method(D_METHOD("set_wish_linear_velocity_position", "position"), &EosEntity::set_wish_linear_velocity_position);
	ClassDB::bind_method(D_METHOD("set_wish_linear_velocity_position_overshot", "position"), &EosEntity::set_wish_linear_velocity_position_overshot);
	ClassDB::bind_method(D_METHOD("set_wish_linear_velocity_absolute", "direction"), &EosEntity::set_wish_linear_velocity_absolute);
	ClassDB::bind_method(D_METHOD("set_wish_linear_velocity_relative", "direction"), &EosEntity::set_wish_linear_velocity_relative);

	ClassDB::bind_method(D_METHOD("set_wish_angular_velocity_keep"), &EosEntity::set_wish_angular_velocity_keep);
	ClassDB::bind_method(D_METHOD("set_wish_angular_velocity_cancel"), &EosEntity::set_wish_angular_velocity_cancel);
	ClassDB::bind_method(D_METHOD("set_wish_angular_velocity_aim", "position"), &EosEntity::set_wish_angular_velocity_aim);
	ClassDB::bind_method(D_METHOD("set_wish_angular_velocity_force", "direction"), &EosEntity::set_wish_angular_velocity_force);

	ClassDB::bind_method(D_METHOD("integrate_forces", "state"), &EosEntity::integrate_forces);
	ClassDB::bind_method(D_METHOD("physics_process"), &EosEntity::physics_process);
}

void EosEntity::compute_stats() {
	// TODO: Use basket
	// TODO: Trigger signal when changing basket
	float readiness_modifier = readiness / base_readiness;

	linear_acceleration = base_linear_acceleration * readiness_modifier;
	angular_acceleration = base_angular_acceleration * readiness_modifier;
	max_linear_velocity = base_max_linear_velocity * readiness_modifier;
	max_angular_velocity = base_max_angular_velocity * readiness_modifier;
}

void EosEntity::set_base_linear_acceleration(double value) {
	base_linear_acceleration = value;
	compute_stats();
}

double EosEntity::get_base_linear_acceleration() const {
	return base_linear_acceleration;
}

void EosEntity::set_base_angular_acceleration(double value) {
	base_angular_acceleration = value;
	compute_stats();
}

double EosEntity::get_base_angular_acceleration() const {
	return base_angular_acceleration;
}

void EosEntity::set_base_max_linear_velocity(double value) {
	base_max_linear_velocity = value;
	compute_stats();
}

double EosEntity::get_base_max_linear_velocity() const {
	return base_max_linear_velocity;
}

void EosEntity::set_base_max_angular_velocity(double value) {
	base_max_angular_velocity = value;
	compute_stats();
}

double EosEntity::get_base_max_angular_velocity() const {
	return base_max_angular_velocity;
}

void EosEntity::set_base_readiness(double value) {
	base_readiness = Math::max((float)value, 1.0f);
	compute_stats();
}

double EosEntity::get_base_readiness() const {
	return base_readiness;
}

void EosEntity::set_readiness(double value) {
	readiness = value;
	compute_stats();
}

double EosEntity::get_readiness() const {
	return readiness;
}

void EosEntity::set_hull_hp(double value) {
	// TODO: Check if destroyed & trigger signal
	hull_hp = value;
}

double EosEntity::get_hull_hp() const {
	return hull_hp;
}

void EosEntity::set_average_armor_hp(double value) {
	// TODO: Update armor grid
	armor_hp = value;
}

double EosEntity::get_average_armor_hp() const {
	// TODO: Compute from armor grid
	return armor_hp;
}

void EosEntity::set_wish_linear_velocity_keep() {
	wish_linear_velocity_type = LINVEL_KEEP;
}

void EosEntity::set_wish_linear_velocity_cancel() {
	wish_linear_velocity_type = LINVEL_CANCEL;
}

void EosEntity::set_wish_linear_velocity_position(Vector2 position) {
	wish_linear_velocity_type = LINVEL_POSITION;
	wish_linear_velocity = position;
}

void EosEntity::set_wish_linear_velocity_position_overshot(Vector2 position) {
	wish_linear_velocity_type = LINVEL_POSITION_OVERSHOT;
	wish_linear_velocity = position;
}

void EosEntity::set_wish_linear_velocity_absolute(Vector2 direction) {
	wish_linear_velocity_type = LINVEL_ABSOLUTE;
	wish_linear_velocity = direction.limit_length(1.0f);
}

void EosEntity::set_wish_linear_velocity_relative(Vector2 direction) {
	wish_linear_velocity_type = LINVEL_RELATIVE;
	wish_linear_velocity = direction.limit_length(1.0f);
}

void EosEntity::set_wish_angular_velocity_keep() {
	wish_angular_velocity_type = ANGVEL_KEEP;
}

void EosEntity::set_wish_angular_velocity_cancel() {
	wish_angular_velocity_type = ANGVEL_CANCEL;
}

void EosEntity::set_wish_angular_velocity_aim(Vector2 position) {
	wish_angular_velocity_type = ANGVEL_AIM;
	wish_angular_velocity = position;
}

void EosEntity::set_wish_angular_velocity_force(double force) {
	wish_angular_velocity_type = ANGVEL_FORCE;
	wish_angular_velocity.x = Math::clamp((float)force, -1.0f, 1.0f);
}

void EosEntity::physics_process() {
}

void EosEntity::integrate_forces(PhysicsDirectBodyState2D *state) {
	float angular_volicity = (float)state->get_angular_velocity();
	Vector2 linear_velocity = state->get_linear_velocity();

	// Angular velocity
	switch (wish_angular_velocity_type) {
		case ANGVEL_KEEP:
			if (angular_volicity > max_angular_velocity) {
				angular_volicity = Math::max(
						AngularVelocityIntegration::stop(
								angular_volicity,
								angular_acceleration,
								DELTA),
						max_angular_velocity);
			} else if (angular_volicity < -max_angular_velocity) {
				angular_volicity = Math::min(
						AngularVelocityIntegration::stop(
								angular_volicity,
								angular_acceleration,
								DELTA),
						max_angular_velocity);
			}
			break;
		case ANGVEL_CANCEL:
			angular_volicity = AngularVelocityIntegration::stop(
					angular_volicity,
					angular_acceleration,
					DELTA);
			break;
		case ANGVEL_AIM:
			angular_volicity = AngularVelocityIntegration::offset(
					get_angle_to(wish_angular_velocity),
					angular_volicity,
					angular_acceleration,
					max_angular_velocity,
					DELTA);
			break;
		case ANGVEL_FORCE:
			angular_volicity = AngularVelocityIntegration::force(
					wish_angular_velocity.x,
					angular_volicity,
					angular_acceleration,
					max_angular_velocity,
					DELTA);
			break;
	}

	// Linear velocity
	switch (wish_linear_velocity_type) {
		case LINVEL_KEEP: {
			float max_linear_velocity_squared = max_linear_velocity * max_linear_velocity;
			if (linear_velocity.length_squared() > max_linear_velocity_squared) {
				Vector2 maybe = LinearVelocityIntegration::stop(
						linear_velocity,
						linear_acceleration,
						DELTA);
				if (maybe.length_squared() < max_linear_velocity_squared) {
					linear_velocity = linear_velocity.normalized() * max_linear_velocity;
				} else {
					linear_velocity = maybe;
				}
			}
			break;
		}
		case LINVEL_CANCEL:
			if (linear_velocity.length_squared() < 1.0f) {
				linear_velocity = Vector2();
			} else {
				linear_velocity = LinearVelocityIntegration::stop(
						linear_velocity,
						linear_acceleration,
						DELTA);
			}
			break;
		case LINVEL_POSITION: {
			Vector2 target = wish_linear_velocity - get_position();
			if (target.length_squared() < 10.0f) {
				// We are alreay on target.
				if (linear_velocity.length_squared() < 1.0f) {
					linear_velocity = Vector2();
				} else {
					linear_velocity = LinearVelocityIntegration::stop(
							linear_velocity,
							linear_acceleration,
							DELTA);
				}
			} else {
				linear_velocity = LinearVelocityIntegration::wish(
						target.limit_length(max_linear_velocity),
						linear_velocity,
						linear_acceleration,
						DELTA);
			}
			break;
		}
		case LINVEL_POSITION_OVERSHOT: {
			Vector2 target = wish_linear_velocity - get_position();
			if (target.length_squared() < 2.0f) {
				linear_velocity = LinearVelocityIntegration::wish(
						Vector2(0.0f, max_linear_velocity),
						linear_velocity,
						linear_acceleration,
						DELTA);
			} else {
				linear_velocity = LinearVelocityIntegration::wish(
						target.normalized() * max_linear_velocity,
						linear_velocity,
						linear_acceleration,
						DELTA);
			}
			break;
		}
		case LINVEL_ABSOLUTE:
			linear_velocity = LinearVelocityIntegration::wish(
					wish_linear_velocity * max_linear_velocity,
					linear_velocity,
					linear_acceleration,
					DELTA);
			break;
		case LINVEL_RELATIVE:
			linear_velocity = LinearVelocityIntegration::wish(
					wish_linear_velocity.rotated(get_rotation()) * max_linear_velocity,
					linear_velocity,
					linear_acceleration,
					DELTA);
			break;
	}

	state->set_angular_velocity(angular_volicity);
	state->set_linear_velocity(linear_velocity);
}

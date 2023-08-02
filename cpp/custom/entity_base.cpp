#include "entity_base.h"
#include "core/object/object.h"
#include "core/variant/variant.h"
#include "preludes.h"
#include "velocity_integration.hpp"

ADD_SETGET_MODIFIERS_IMPL(EntityBase, f32, linear_acceleration)
ADD_SETGET_MODIFIERS_IMPL(EntityBase, f32, max_linear_velocity)
ADD_SETGET_MODIFIERS_IMPL(EntityBase, f32, angular_acceleration)
ADD_SETGET_MODIFIERS_IMPL(EntityBase, f32, max_angular_velocity)

ADD_SETGET_MODIFIERS_IMPL(EntityBase, f32, max_hull_hp)
ADD_SETGET_MODIFIERS_IMPL(EntityBase, f32, max_armor_hp)

void EntityBase::_bind_methods() {
	ADD_SETGET_MODIFIERS_PROPERTY(EntityBase, linear_acceleration)
	ADD_SETGET_MODIFIERS_PROPERTY(EntityBase, max_linear_velocity)
	ADD_SETGET_MODIFIERS_PROPERTY(EntityBase, angular_acceleration)
	ADD_SETGET_MODIFIERS_PROPERTY(EntityBase, max_angular_velocity)

	ADD_SETGET_MODIFIERS_PROPERTY(EntityBase, max_hull_hp)
	ADD_SETGET_MODIFIERS_PROPERTY(EntityBase, max_armor_hp)

	ClassDB::bind_method(D_METHOD("wish_linvel_keep"), &EntityBase::wish_linvel_keep);
	ClassDB::bind_method(D_METHOD("wish_linvel_cancel"), &EntityBase::wish_linvel_stop);
	ClassDB::bind_method(D_METHOD("wish_linvel_position", "pos"), &EntityBase::wish_linvel_position);
	ClassDB::bind_method(D_METHOD("wish_linvel_position_overshoot", "pos"), &EntityBase::wish_linvel_position_overshoot);
	ClassDB::bind_method(D_METHOD("wish_linvel_absolute", "direction"), &EntityBase::wish_linvel_absolute);
	ClassDB::bind_method(D_METHOD("wish_linvel_relative", "direction"), &EntityBase::wish_linvel_relative);

	ClassDB::bind_method(D_METHOD("wish_angvel_keep"), &EntityBase::wish_angvel_keep);
	ClassDB::bind_method(D_METHOD("wish_angvel_cancel"), &EntityBase::wish_angvel_stop);
	ClassDB::bind_method(D_METHOD("wish_angvel_aim", "pos"), &EntityBase::wish_angvel_aim);
	ClassDB::bind_method(D_METHOD("wish_angvel_force", "direction"), &EntityBase::wish_angvel_force);

	ClassDB::bind_method(D_METHOD("_base_integrate_forces", "state"), &EntityBase::_base_integrate_forces);
}

void EntityBase::wish_linvel_keep() {
	wish_linvel_type = LINVEL_KEEP;
}

void EntityBase::wish_linvel_stop() {
	wish_linvel_type = LINVEL_STOP;
}

void EntityBase::wish_linvel_position(Vector2 pos) {
	wish_linvel_type = LINVEL_POSITION;
	wish_linvel = pos;
}

void EntityBase::wish_linvel_position_overshoot(Vector2 pos) {
	wish_linvel_type = LINVEL_POSITION_OVERSHOOT;
	wish_linvel = pos;
}

void EntityBase::wish_linvel_absolute(Vector2 direction) {
	wish_linvel_type = LINVEL_ABSOLUTE;
	wish_linvel = direction.limit_length(1.0f);
}

void EntityBase::wish_linvel_relative(Vector2 direction) {
	wish_linvel_type = LINVEL_RELATIVE;
	wish_linvel = direction.limit_length(1.0f);
}

void EntityBase::wish_angvel_keep() {
	wish_angvel_type = ANGVEL_KEEP;
}

void EntityBase::wish_angvel_stop() {
	wish_angvel_type = ANGVEL_STOP;
}

void EntityBase::wish_angvel_aim(Vector2 pos) {
	wish_angvel_type = ANGVEL_AIM;
	wish_angvel = pos;
}

void EntityBase::wish_angvel_force(f32 force) {
	wish_angvel_type = ANGVEL_FORCE;
	wish_angvel.x = CLAMP(force, -1.0f, 1.0f);
}

void EntityBase::_base_integrate_forces(PhysicsDirectBodyState2D *state) {
	if (!this->is_using_custom_integrator()) {
		return;
	}

	// TODO: get this
	f32 delta = 1.0f / 60.0f;

	// Angular velocity
	f32 angvel = (f32)state->get_angular_velocity();
	f32 angacc = this->get_angular_acceleration();
	f32 max_angvel = this->get_max_angular_velocity();
	switch (wish_angvel_type) {
		case ANGVEL_KEEP:
			if (abs(angvel) > max_angvel) {
				angvel = VelocityIntegration::stop_angvel(
						angvel,
						angacc,
						delta);
				if (abs(angvel) < max_angvel) {
					angvel = SIGN(angvel) * max_angvel;
				}
			}
			break;
		case ANGVEL_STOP:
			angvel = VelocityIntegration::stop_angvel(
					angvel,
					angacc,
					delta);
			break;
		case ANGVEL_AIM: {
			f32 wish_angle_change = get_angle_to(wish_angvel);
			f32 wish_dir = SIGN(wish_angle_change);

			f32 close_smooth = MIN(ABS(wish_angle_change), 0.2f) / 0.2f;
			close_smooth *= close_smooth * close_smooth;

			if (wish_dir != SIGN(angvel)) {
				angvel = VelocityIntegration::angvel(
						wish_dir * max_angvel * close_smooth,
						angvel,
						angacc,
						delta);
			} else {
				f32 time_to_target = ABS(wish_angle_change / angvel);
				f32 time_to_stop = ABS(angvel / angacc);

				if (time_to_target > time_to_stop) {
					angvel = VelocityIntegration::angvel(
							wish_dir * max_angvel * close_smooth,
							angvel,
							angacc,
							delta);
				} else {
					angvel = VelocityIntegration::angvel(
							-wish_dir * max_angvel * close_smooth,
							angvel,
							angacc,
							delta);
				}
			}
		} break;
		case ANGVEL_FORCE:
			angvel = VelocityIntegration::angvel(
					wish_angvel.x * max_angvel,
					angvel,
					angacc,
					delta);
			break;
	}

	// Linear velocity
	Vector2 linvel = state->get_linear_velocity();
	f32 linacc = this->get_linear_acceleration();
	f32 max_linvel = this->get_max_linear_velocity();
	switch (wish_linvel_type) {
		case LINVEL_KEEP: {
			float max_linvel_squared = max_linvel * max_linvel;
			if (linvel.length_squared() > max_linvel_squared) {
				linvel = VelocityIntegration::stop_linvel(
						linvel,
						linacc,
						delta);
				if (linvel.length_squared() < max_linvel_squared && !linvel.is_zero_approx()) {
					linvel = linvel.normalized() * max_linvel;
				}
			}
			break;
		}
		case LINVEL_STOP:
			if (linvel.length_squared() < 10.0f) {
				linvel = Vector2();
			} else {
				linvel = VelocityIntegration::stop_linvel(
						linvel,
						linacc,
						delta);
			}
			break;
		case LINVEL_POSITION: {
			Vector2 target = wish_linvel - get_position();
			if (target.length_squared() < 100.0f) {
				// We are alreay on target.
				if (linvel.length_squared() < 1.0f) {
					linvel = Vector2();
				} else {
					linvel = VelocityIntegration::stop_linvel(
							linvel,
							linacc,
							delta);
				}
			} else {
				linvel = VelocityIntegration::linvel(
						target.limit_length(max_linvel),
						linvel,
						linacc,
						delta);
			}
			break;
		}
		case LINVEL_POSITION_OVERSHOOT: {
			Vector2 target = wish_linvel - get_position();
			if (target.length_squared() < 1.0f) {
				linvel = VelocityIntegration::linvel(
						Vector2(0.0f, max_linvel),
						linvel,
						linacc,
						delta);
			} else {
				linvel = VelocityIntegration::linvel(
						target.normalized() * max_linvel,
						linvel,
						linacc,
						delta);
			}
			break;
		}
		case LINVEL_ABSOLUTE:
			linvel = VelocityIntegration::linvel(
					wish_linvel * max_linvel,
					linvel,
					linacc,
					delta);
			break;
		case LINVEL_RELATIVE:
			linvel = VelocityIntegration::linvel(
					wish_linvel.rotated(get_rotation()) * max_linvel,
					linvel,
					linacc,
					delta);
			break;
	}

	state->set_angular_velocity(angvel);
	state->set_linear_velocity(linvel);
}
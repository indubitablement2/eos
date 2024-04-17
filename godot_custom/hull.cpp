#include "hull.h"
#include "core/config/engine.h"
#include "core/math/vector2.h"
#include "core/object/object.h"
#include "core/typedefs.h"
#include "core/variant/variant.h"

void HullData::_bind_methods() {
	SET_GET_BIND(armor_cells_max, PackedByteArray, HullData);
	ADD_PROPERTY(PropertyInfo(Variant::PACKED_BYTE_ARRAY, "armor_cells_max"), "set_armor_cells_max", "get_armor_cells_max");

	DATA_PROP_BIND(linear_acceleration);
	DATA_PROP_BIND(linear_velocity_max);
	DATA_PROP_BIND(angular_acceleration);
	DATA_PROP_BIND(angular_velocity_max);
	DATA_PROP_BIND(armor_max);
	DATA_PROP_BIND(hull_max);
}

SET_GET_IMPL(armor_cells_max, PackedByteArray, HullData);

DATA_PROP_IMPL(linear_acceleration);
DATA_PROP_IMPL(linear_velocity_max);
DATA_PROP_IMPL(angular_acceleration);
DATA_PROP_IMPL(angular_velocity_max);
DATA_PROP_IMPL(armor_max);
DATA_PROP_IMPL(hull_max);

void Hull::_bind_methods() {
	SET_GET_BIND(data, Ref<HullBaseData>, Hull);
	SET_GET_BIND(wish_linear_velocity_type, WishLinearVelocityType, Hull);
	SET_GET_BIND(wish_linear_velocity, Vector2, Hull);
	SET_GET_BIND(wish_angular_velocity_type, WishAngularVelocityType, Hull);
	SET_GET_BIND(wish_angular_velocity, Vector2, Hull);
	SET_GET_BIND(turrets, TypedArray<Node2D>, Hull);
	SET_GET_BIND(armor_cells, PackedByteArray, Hull);
	SET_GET_BIND(hull_relative, f32, Hull);

	PROP_BIND(linear_acceleration);
	PROP_BIND(linear_velocity_max);
	PROP_BIND(angular_acceleration);
	PROP_BIND(angular_velocity_max);
	PROP_BIND(armor_max);
	PROP_BIND(hull_max);

	ADD_PROPERTY(PropertyInfo(Variant::OBJECT, "data", PROPERTY_HINT_RESOURCE_TYPE, "HullBaseData"), "set_data", "get_data");
	ADD_PROPERTY(PropertyInfo(Variant::INT, "wish_linear_velocity_type", PROPERTY_HINT_ENUM, "None,Keep,Cancel,PositionSmooth,PositionOvershoot,ForceAbsolute,ForceRelative"), "set_wish_linear_velocity_type", "get_wish_linear_velocity_type");
	ADD_PROPERTY(PropertyInfo(Variant::VECTOR2, "wish_linear_velocity"), "set_wish_linear_velocity", "get_wish_linear_velocity");
	ADD_PROPERTY(PropertyInfo(Variant::INT, "wish_angular_velocity_type", PROPERTY_HINT_ENUM, "None,Keep,Cancel,AimSmooth,Force"), "set_wish_angular_velocity_type", "get_wish_angular_velocity_type");
	ADD_PROPERTY(PropertyInfo(Variant::VECTOR2, "wish_angular_velocity"), "set_wish_angular_velocity", "get_wish_angular_velocity");
	ADD_PROPERTY(PropertyInfo(Variant::ARRAY, "turrets", PROPERTY_HINT_ARRAY_TYPE, "Node2D"), "set_turrets", "get_turrets");
	ADD_PROPERTY(PropertyInfo(Variant::PACKED_BYTE_ARRAY, "armor_cells"), "set_armor_cells", "get_armor_cells");
	ADD_PROPERTY(PropertyInfo(Variant::FLOAT, "hull_relative"), "set_hull_relative", "get_hull_relative");

	// ADD_SIGNAL(MethodInfo("body_shape_entered", PropertyInfo(Variant::RID, "body_rid"), PropertyInfo(Variant::OBJECT, "body", PROPERTY_HINT_RESOURCE_TYPE, "Node"), PropertyInfo(Variant::INT, "body_shape_index"), PropertyInfo(Variant::INT, "local_shape_index")));

	BIND_ENUM_CONSTANT(WISH_LINEAR_VELOCITY_TYPE_NONE);
	BIND_ENUM_CONSTANT(WISH_LINEAR_VELOCITY_TYPE_KEEP);
	BIND_ENUM_CONSTANT(WISH_LINEAR_VELOCITY_TYPE_CANCEL);
	BIND_ENUM_CONSTANT(WISH_LINEAR_VELOCITY_TYPE_POSITION_SMOOTH);
	BIND_ENUM_CONSTANT(WISH_LINEAR_VELOCITY_TYPE_POSITION_OVERSHOOT);
	BIND_ENUM_CONSTANT(WISH_LINEAR_VELOCITY_TYPE_FORCE_ABSOLUTE);
	BIND_ENUM_CONSTANT(WISH_LINEAR_VELOCITY_TYPE_FORCE_RELATIVE);

	BIND_ENUM_CONSTANT(WISH_ANGULAR_VELOCITY_TYPE_NONE);
	BIND_ENUM_CONSTANT(WISH_ANGULAR_VELOCITY_TYPE_KEEP);
	BIND_ENUM_CONSTANT(WISH_ANGULAR_VELOCITY_TYPE_CANCEL);
	BIND_ENUM_CONSTANT(WISH_ANGULAR_VELOCITY_TYPE_AIM_SMOOTH);
	BIND_ENUM_CONSTANT(WISH_ANGULAR_VELOCITY_TYPE_FORCE);
}

SET_GET_IMPL(data, Ref<HullData>, Hull);
SET_GET_IMPL(wish_linear_velocity_type, WishLinearVelocityType, Hull);
SET_GET_IMPL(wish_linear_velocity, Vector2, Hull);
SET_GET_IMPL(wish_angular_velocity_type, WishAngularVelocityType, Hull);
SET_GET_IMPL(wish_angular_velocity, Vector2, Hull);
SET_GET_IMPL(turrets, TypedArray<Node2D>, Hull);
SET_GET_IMPL(armor_cells, PackedByteArray, Hull);
SET_GET_IMPL(hull_relative, f32, Hull);

PROP_IMPL(linear_acceleration);
PROP_IMPL(linear_velocity_max);
PROP_IMPL(angular_acceleration);
PROP_IMPL(angular_velocity_max);
PROP_IMPL(armor_max);
PROP_IMPL(hull_max);

void Hull::_notification(int p_what) {
	if (Engine::get_singleton()->is_editor_hint()) {
		return;
	}

	switch (p_what) {
		case NOTIFICATION_PHYSICS_PROCESS:
			_apply_wish_movement();
			break;
	}
}

void Hull::_apply_wish_movement() {
	switch (wish_linear_velocity_type) {
		case WISH_LINEAR_VELOCITY_TYPE_NONE: {
			thruster_linear = Vector2();
		} break;
		case WISH_LINEAR_VELOCITY_TYPE_KEEP: {
			Vector2 force = (get_linear_velocity().limit_length(linear_velocity_max_get()) - get_linear_velocity()).limit_length(linear_acceleration_get());
			if (abs(force.x) > 1.0 || abs(force.y) > 1.0) {
				apply_central_force(force);
			}

			thruster_linear = Vector2();
		} break;
		case WISH_LINEAR_VELOCITY_TYPE_CANCEL: {
			Vector2 force = -get_linear_velocity().limit_length(linear_acceleration_get());
			if (abs(force.x) > 1.0 || abs(force.y) > 1.0) {
				apply_central_force(force);
			}

			thruster_linear = force;
		} break;
		case WISH_LINEAR_VELOCITY_TYPE_POSITION_SMOOTH: {
			Vector2 force = wish_linear_velocity - get_position();
			if (force.length_squared() < 256.0) {
				force = -get_linear_velocity().limit_length(linear_acceleration_get());
				// thruster_linear = Vector2();
			} else {
				force = (force.limit_length(linear_velocity_max_get()) - get_linear_velocity()).limit_length(linear_acceleration_get());
				// thruster_linear = force;
			}
			if (abs(force.x) > 1.0 || abs(force.y) > 1.0) {
				apply_central_force(force);
			}

			thruster_linear = force;

		} break;
		case WISH_LINEAR_VELOCITY_TYPE_POSITION_OVERSHOOT: {
			Vector2 force = get_position().direction_to(wish_linear_velocity);
			if (force.length_squared() < 0.5) {
				force = Vector2(0.0, -1.0);
			}
			force = (force * linear_velocity_max_get() - get_linear_velocity()).limit_length(linear_acceleration_get());
			if (abs(force.x) > 1.0 || abs(force.y) > 1.0) {
				apply_central_force(force);
			}

			thruster_linear = force;
		} break;
		case WISH_LINEAR_VELOCITY_TYPE_FORCE_ABSOLUTE: {
			wish_linear_velocity = wish_linear_velocity.limit_length(1.0);
			f32 linacc = linear_acceleration_get();

			Vector2 force = (wish_linear_velocity * linear_velocity_max_get() - get_linear_velocity()).limit_length(linacc);
			if (abs(force.x) > 1.0 || abs(force.y) > 1.0) {
				apply_central_force(force);
			}

			thruster_linear = wish_linear_velocity * linacc;
		} break;
		case WISH_LINEAR_VELOCITY_TYPE_FORCE_RELATIVE: {
			wish_linear_velocity = wish_linear_velocity.limit_length(1.0);
			f32 linacc = linear_acceleration_get();

			Vector2 rot = wish_linear_velocity.rotated(get_rotation());
			Vector2 force = (rot * linear_velocity_max_get() - get_linear_velocity()).limit_length(linacc);
			if (abs(force.x) > 1.0 || abs(force.y) > 1.0) {
				apply_central_force(force);
			}

			thruster_linear = rot * linacc;
		} break;
	}

	switch (wish_angular_velocity_type) {
		case WISH_ANGULAR_VELOCITY_TYPE_NONE: {
			thruster_angular = 0.0;
		} break;
		case WISH_ANGULAR_VELOCITY_TYPE_KEEP: {
			f32 angular_velocity_max = angular_velocity_max_get();
			f32 angular_acceleration = angular_acceleration_get();

			if (abs(get_angular_velocity()) > angular_velocity_max) {
				f32 force = CLAMP(get_angular_velocity(), -angular_velocity_max, angular_velocity_max);
				force = CLAMP(force - get_angular_velocity(), -angular_acceleration, angular_acceleration);
				apply_torque(force);
			}

			thruster_angular = 0.0;
		} break;
		case WISH_ANGULAR_VELOCITY_TYPE_CANCEL: {
			f32 angular_acceleration = angular_acceleration_get();

			f32 force = CLAMP(-get_angular_velocity(), -angular_acceleration, angular_acceleration);
			if (abs(force) > 0.01) {
				apply_torque(force);
			}

			thruster_angular = force;
		} break;
		case WISH_ANGULAR_VELOCITY_TYPE_AIM_SMOOTH: {
			f32 angular_velocity_max = angular_velocity_max_get();
			f32 angular_acceleration = angular_acceleration_get();

			f32 offset = to_local(wish_angular_velocity).angle();
			f32 wish_dir = 1.0;
			if (offset < 0.0) {
				wish_dir = -1.0;
			}
			f32 angvel_dir = 1.0;
			if (get_angular_velocity() < 0.0) {
				angvel_dir = -1.0;
			}

			f32 close_smooth = MIN(abs(offset), 0.2) / 0.2;
			close_smooth *= close_smooth * close_smooth;

			if (wish_dir == angvel_dir) {
				f32 time_to_target = abs(offset / get_angular_velocity());
				f32 time_to_stop = abs(get_angular_velocity() / angular_acceleration);

				if (time_to_target < time_to_stop) {
					close_smooth *= -1.0;
				}
			}

			f32 force = CLAMP(wish_dir * angular_velocity_max * close_smooth - get_angular_velocity(), -angular_acceleration, angular_acceleration);
			if (abs(force) > 0.01) {
				apply_torque(force);
			}

			thruster_angular = force;
		} break;
		case WISH_ANGULAR_VELOCITY_TYPE_FORCE: {
			wish_angular_velocity.x = CLAMP(wish_angular_velocity.x, -1.0, 1.0);

			f32 angular_acceleration = angular_acceleration_get();

			f32 force = CLAMP(wish_angular_velocity.x * angular_velocity_max_get() - get_angular_velocity(), -angular_acceleration, angular_acceleration);
			if (abs(force) > 0.01) {
				apply_torque(force);
			}

			thruster_angular = wish_angular_velocity.x * angular_acceleration;
		} break;
	}
}

Hull::Hull() {
	set_center_of_mass_mode(CenterOfMassMode::CENTER_OF_MASS_MODE_CUSTOM);
	set_physics_process(true);
	// set_use_custom_integrator(true);
}
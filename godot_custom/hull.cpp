#include "hull.h"
#include "core/config/engine.h"
#include "core/math/vector2.h"
#include "core/object/object.h"
#include "core/typedefs.h"
#include "core/variant/variant.h"
#include "scene/main/node.h"
#include "servers/physics_server_2d.h"

const f32 DT = 0.1;

void HullData::_bind_methods() {
	DATA_PROP_BIND(linear_acceleration, "0,1000,or_greater,suffix:p/s²");
	DATA_PROP_BIND(linear_velocity_max, "0,1000,or_greater,suffix:p/s");
	DATA_PROP_BIND(angular_acceleration, "0,100,or_greater,suffix:rad/s²");
	DATA_PROP_BIND(angular_velocity_max, "0,100,or_greater,suffix:rad/s");

	SET_GET_BIND(armor_cells_max, PackedByteArray, HullData);
	ADD_PROPERTY(PropertyInfo(Variant::PACKED_BYTE_ARRAY, "armor_cells_max"), "set_armor_cells_max", "get_armor_cells_max");
	DATA_PROP_BIND(armor_max, "");
	DATA_PROP_BIND(hull_max, "");

	SET_GET_BIND(num_turrets, i32, HullData);
	ADD_PROPERTY(PropertyInfo(Variant::INT, "num_turrets"), "set_num_turrets", "get_num_turrets");
}

SET_GET_IMPL(armor_cells_max, PackedByteArray, HullData);
SET_GET_IMPL(num_turrets, i32, HullData);

DATA_PROP_IMPL(linear_acceleration);
DATA_PROP_IMPL(linear_velocity_max);
DATA_PROP_IMPL(angular_acceleration);
DATA_PROP_IMPL(angular_velocity_max);
DATA_PROP_IMPL(armor_max);
DATA_PROP_IMPL(hull_max);

void Hull::_bind_methods() {
	SET_GET_BIND(data, Ref<HullData>, Hull);
	SET_GET_BIND(wish_linear_velocity_type, WishLinearVelocityType, Hull);
	SET_GET_BIND(wish_linear_velocity, Vector2, Hull);
	SET_GET_BIND(wish_angular_velocity_type, WishAngularVelocityType, Hull);
	SET_GET_BIND(wish_angular_velocity, Vector2, Hull);
	SET_GET_BIND(armor_cells, PackedByteArray, Hull);
	SET_GET_BIND(hull_relative, f32, Hull);

	PROP_BIND(linear_acceleration);
	PROP_BIND(linear_velocity_max);
	PROP_BIND(angular_acceleration);
	PROP_BIND(angular_velocity_max);
	PROP_BIND(armor_max);
	PROP_BIND(hull_max);

	ADD_PROPERTY(PropertyInfo(Variant::OBJECT, "data", PROPERTY_HINT_RESOURCE_TYPE, "HullData"), "set_data", "get_data");
	ADD_PROPERTY(PropertyInfo(Variant::INT, "wish_linear_velocity_type", PROPERTY_HINT_ENUM, "Keep,Cancel,PositionSmooth,PositionOvershoot,ForceAbsolute,ForceRelative,SafeVelocity"), "set_wish_linear_velocity_type", "get_wish_linear_velocity_type");
	ADD_PROPERTY(PropertyInfo(Variant::VECTOR2, "wish_linear_velocity"), "set_wish_linear_velocity", "get_wish_linear_velocity");
	ADD_PROPERTY(PropertyInfo(Variant::INT, "wish_angular_velocity_type", PROPERTY_HINT_ENUM, "Keep,Cancel,AimSmooth,Force"), "set_wish_angular_velocity_type", "get_wish_angular_velocity_type");
	ADD_PROPERTY(PropertyInfo(Variant::VECTOR2, "wish_angular_velocity"), "set_wish_angular_velocity", "get_wish_angular_velocity");
	ADD_PROPERTY(PropertyInfo(Variant::PACKED_BYTE_ARRAY, "armor_cells"), "set_armor_cells", "get_armor_cells");
	ADD_PROPERTY(PropertyInfo(Variant::FLOAT, "hull_relative"), "set_hull_relative", "get_hull_relative");

	// ADD_SIGNAL(MethodInfo("body_shape_entered", PropertyInfo(Variant::RID, "body_rid"), PropertyInfo(Variant::OBJECT, "body", PROPERTY_HINT_RESOURCE_TYPE, "Node"), PropertyInfo(Variant::INT, "body_shape_index"), PropertyInfo(Variant::INT, "local_shape_index")));

	BIND_ENUM_CONSTANT(WISH_LINEAR_VELOCITY_TYPE_KEEP);
	BIND_ENUM_CONSTANT(WISH_LINEAR_VELOCITY_TYPE_CANCEL);
	BIND_ENUM_CONSTANT(WISH_LINEAR_VELOCITY_TYPE_POSITION_SMOOTH);
	BIND_ENUM_CONSTANT(WISH_LINEAR_VELOCITY_TYPE_POSITION_OVERSHOOT);
	BIND_ENUM_CONSTANT(WISH_LINEAR_VELOCITY_TYPE_FORCE_ABSOLUTE);
	BIND_ENUM_CONSTANT(WISH_LINEAR_VELOCITY_TYPE_FORCE_RELATIVE);
	BIND_ENUM_CONSTANT(WISH_LINEAR_VELOCITY_TYPE_SAVE_VELOCITY);

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
		// case NOTIFICATION_READY: {
		// 	set_physics_process(true);
		// } break;
		case NOTIFICATION_PHYSICS_PROCESS: {
			PhysicsDirectBodyState2D *state = PhysicsServer2D::get_singleton()->body_get_direct_state(get_rid());
			apply_wish_movement(state);
		} break;
	}
}

void Hull::apply_wish_movement(PhysicsDirectBodyState2D *state) {
	const Vector2 linvel = state->get_linear_velocity();
	Vector2 wish_linvel;
	switch (wish_linear_velocity_type) {
		case WISH_LINEAR_VELOCITY_TYPE_KEEP: {
			wish_linvel = linvel.limit_length(linear_velocity_max_get());
		} break;
		case WISH_LINEAR_VELOCITY_TYPE_CANCEL: {
			wish_linvel = Vector2();
		} break;
		case WISH_LINEAR_VELOCITY_TYPE_POSITION_SMOOTH: {
			wish_linvel = wish_linear_velocity - get_position();
			if (wish_linvel.length_squared() < 256.0) {
				wish_linvel = Vector2();
			} else {
				wish_linvel = wish_linvel.limit_length(linear_velocity_max_get());
			}
		} break;
		case WISH_LINEAR_VELOCITY_TYPE_POSITION_OVERSHOOT: {
			wish_linvel = get_position().direction_to(wish_linear_velocity);
			if (wish_linvel.length_squared() < 0.5) {
				wish_linvel = Vector2(0.0, -1.0);
			}
			wish_linvel *= linear_velocity_max_get();
		} break;
		case WISH_LINEAR_VELOCITY_TYPE_FORCE_ABSOLUTE: {
			wish_linvel = wish_linear_velocity.limit_length(1.0) * linear_velocity_max_get();
		} break;
		case WISH_LINEAR_VELOCITY_TYPE_FORCE_RELATIVE: {
			wish_linvel = wish_linear_velocity.limit_length(1.0).rotated(get_rotation()) * linear_velocity_max_get();
		} break;
		case WISH_LINEAR_VELOCITY_TYPE_SAVE_VELOCITY: {
			wish_linvel = wish_linear_velocity.limit_length(linear_velocity_max_get());
		} break;
	}

	Vector2 linvel_change = (wish_linvel - linvel).limit_length(linear_acceleration_get() * DT);
	if (abs(linvel_change.x) > 0.1 || abs(linvel_change.y) > 0.1) {
		state->set_linear_velocity(linvel + linvel_change);
	}
	thruster_linear = wish_linvel;

	const f32 angvel = state->get_angular_velocity();
	f32 angvel_max = angular_velocity_max_get();
	f32 wish_angvel = 0.0;
	switch (wish_angular_velocity_type) {
		case WISH_ANGULAR_VELOCITY_TYPE_KEEP: {
			wish_angvel = CLAMP(angvel, -angvel_max, angvel_max);
		} break;
		case WISH_ANGULAR_VELOCITY_TYPE_CANCEL: {
			wish_angvel = 0.0;
		} break;
		case WISH_ANGULAR_VELOCITY_TYPE_AIM_SMOOTH: {
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
				f32 time_to_stop = abs(get_angular_velocity() / angular_acceleration_get());

				if (time_to_target < time_to_stop) {
					close_smooth *= -1.0;
				}
			}

			wish_angvel = wish_dir * angular_velocity_max_get() * close_smooth;
		} break;
		case WISH_ANGULAR_VELOCITY_TYPE_FORCE: {
			wish_angvel = CLAMP(wish_angular_velocity.x, -1.0, 1.0) * angular_velocity_max_get();
		} break;
	}

	f32 angacc_dt = angular_acceleration_get() * DT;
	f32 angvel_change = CLAMP(wish_angvel - angvel, -angacc_dt, angacc_dt);
	if (abs(angvel_change) > 0.01) {
		state->set_angular_velocity(angvel + angvel_change);
	}
	thruster_angular = wish_angvel;
}

Hull::Hull() {
	set_center_of_mass_mode(CenterOfMassMode::CENTER_OF_MASS_MODE_CUSTOM);
	set_use_custom_integrator(true);
}
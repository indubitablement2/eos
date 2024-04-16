#include "hull_base.h"
#include "core/config/engine.h"
#include "core/math/vector2.h"
#include "core/object/object.h"
#include "core/typedefs.h"
#include "core/variant/variant.h"

void HullBase::_bind_methods() {
	ClassDB::bind_method(D_METHOD("set_wish_linear_velocity_type"), &HullBase::set_wish_linear_velocity_type);
	ClassDB::bind_method(D_METHOD("get_wish_linear_velocity_type"), &HullBase::get_wish_linear_velocity_type);
	ClassDB::bind_method(D_METHOD("set_wish_linear_velocity"), &HullBase::set_wish_linear_velocity);
	ClassDB::bind_method(D_METHOD("get_wish_linear_velocity"), &HullBase::get_wish_linear_velocity);
	ClassDB::bind_method(D_METHOD("set_wish_angular_velocity_type"), &HullBase::set_wish_angular_velocity_type);
	ClassDB::bind_method(D_METHOD("get_wish_angular_velocity_type"), &HullBase::get_wish_angular_velocity_type);
	ClassDB::bind_method(D_METHOD("set_wish_angular_velocity"), &HullBase::set_wish_angular_velocity);
	ClassDB::bind_method(D_METHOD("get_wish_angular_velocity"), &HullBase::get_wish_angular_velocity);

	ClassDB::bind_method(D_METHOD("set_linear_acceleration"), &HullBase::set_linear_acceleration);
	ClassDB::bind_method(D_METHOD("get_linear_acceleration"), &HullBase::get_linear_acceleration);
	ClassDB::bind_method(D_METHOD("set_linear_velocity_max"), &HullBase::set_linear_velocity_max);
	ClassDB::bind_method(D_METHOD("get_linear_velocity_max"), &HullBase::get_linear_velocity_max);
	ClassDB::bind_method(D_METHOD("set_angular_acceleration"), &HullBase::set_angular_acceleration);
	ClassDB::bind_method(D_METHOD("get_angular_acceleration"), &HullBase::get_angular_acceleration);
	ClassDB::bind_method(D_METHOD("set_angular_velocity_max"), &HullBase::set_angular_velocity_max);
	ClassDB::bind_method(D_METHOD("get_angular_velocity_max"), &HullBase::get_angular_velocity_max);

	ClassDB::bind_method(D_METHOD("set_turrets"), &HullBase::set_turrets);
	ClassDB::bind_method(D_METHOD("get_turrets"), &HullBase::get_turrets);

	ClassDB::bind_method(D_METHOD("set_armor_cells"), &HullBase::set_armor_cells);
	ClassDB::bind_method(D_METHOD("get_armor_cells"), &HullBase::get_armor_cells);
	ClassDB::bind_method(D_METHOD("set_armor_cells_max"), &HullBase::set_armor_cells_max);
	ClassDB::bind_method(D_METHOD("get_armor_cells_max"), &HullBase::get_armor_cells_max);
	ClassDB::bind_method(D_METHOD("set_armor_max"), &HullBase::set_armor_max);
	ClassDB::bind_method(D_METHOD("get_armor_max"), &HullBase::get_armor_max);

	ClassDB::bind_method(D_METHOD("set_hull"), &HullBase::set_hull);
	ClassDB::bind_method(D_METHOD("get_hull"), &HullBase::get_hull);
	ClassDB::bind_method(D_METHOD("set_hull_max"), &HullBase::set_hull_max);
	ClassDB::bind_method(D_METHOD("get_hull_max"), &HullBase::get_hull_max);

	ADD_PROPERTY(PropertyInfo(Variant::ARRAY, "turrets"), "set_turrets", "get_turrets");
	ADD_GROUP("Wish Velocity", "wish_");
	ADD_PROPERTY(PropertyInfo(Variant::INT, "wish_linear_velocity_type"), "set_wish_linear_velocity_type", "get_wish_linear_velocity_type");
	ADD_PROPERTY(PropertyInfo(Variant::VECTOR2, "wish_linear_velocity"), "set_wish_linear_velocity", "get_wish_linear_velocity");
	ADD_PROPERTY(PropertyInfo(Variant::INT, "wish_angular_velocity_type"), "set_wish_angular_velocity_type", "get_wish_angular_velocity_type");
	ADD_PROPERTY(PropertyInfo(Variant::VECTOR2, "wish_angular_velocity"), "set_wish_angular_velocity", "get_wish_angular_velocity");
	ADD_GROUP("Movement", "");
	ADD_PROPERTY(PropertyInfo(Variant::FLOAT, "linear_acceleration"), "set_linear_acceleration", "get_linear_acceleration");
	ADD_PROPERTY(PropertyInfo(Variant::FLOAT, "linear_velocity_max"), "set_linear_velocity_max", "get_linear_velocity_max");
	ADD_PROPERTY(PropertyInfo(Variant::FLOAT, "angular_acceleration"), "set_angular_acceleration", "get_angular_acceleration");
	ADD_PROPERTY(PropertyInfo(Variant::FLOAT, "angular_velocity_max"), "set_angular_velocity_max", "get_angular_velocity_max");
	ADD_GROUP("Armor", "");
	ADD_PROPERTY(PropertyInfo(Variant::PACKED_BYTE_ARRAY, "armor_cells"), "set_armor_cells", "get_armor_cells");
	ADD_PROPERTY(PropertyInfo(Variant::PACKED_BYTE_ARRAY, "armor_cells_max"), "set_armor_cells_max", "get_armor_cells_max");
	ADD_PROPERTY(PropertyInfo(Variant::FLOAT, "armor_max"), "set_armor_max", "get_armor_max");
	ADD_GROUP("Hull", "");
	ADD_PROPERTY(PropertyInfo(Variant::FLOAT, "hull"), "set_hull", "get_hull");
	ADD_PROPERTY(PropertyInfo(Variant::FLOAT, "hull_max"), "set_hull_max", "get_hull_max");

	BIND_ENUM_CONSTANT(LINVEL_NONE);
	BIND_ENUM_CONSTANT(LINVEL_KEEP);
	BIND_ENUM_CONSTANT(LINVEL_CANCEL);
	BIND_ENUM_CONSTANT(LINVEL_POSITION_SMOOTH);
	BIND_ENUM_CONSTANT(LINVEL_POSITION_OVERSHOOT);
	BIND_ENUM_CONSTANT(LINVEL_FORCE_ABSOLUTE);
	BIND_ENUM_CONSTANT(LINVEL_FORCE_RELATIVE);

	BIND_ENUM_CONSTANT(ANGVEL_NONE);
	BIND_ENUM_CONSTANT(ANGVEL_KEEP);
	BIND_ENUM_CONSTANT(ANGVEL_CANCEL);
	BIND_ENUM_CONSTANT(ANGVEL_AIM_SMOOTH);
	BIND_ENUM_CONSTANT(ANGVEL_FORCE);
}

void HullBase::_notification(int p_what) {
	if (Engine::get_singleton()->is_editor_hint()) {
		return;
	}

	switch (p_what) {
		case NOTIFICATION_PHYSICS_PROCESS:
			_apply_wish_movement();
			break;
	}
}

void HullBase::_apply_wish_movement() {
	switch (wish_linvel_type) {
		case LINVEL_NONE: {
			thruster_linear = Vector2();
		} break;
		case LINVEL_KEEP: {
			Vector2 force = (get_linear_velocity().limit_length(linear_velocity_max) - get_linear_velocity()).limit_length(linear_acceleration);
			if (abs(force.x) > 1.0 || abs(force.y) > 1.0) {
				apply_central_force(force);
			}

			thruster_linear = Vector2();
		} break;
		case LINVEL_CANCEL: {
			Vector2 force = -get_linear_velocity().limit_length(linear_acceleration);
			if (abs(force.x) > 1.0 || abs(force.y) > 1.0) {
				apply_central_force(force);
			}

			thruster_linear = force;
		} break;
		case LINVEL_POSITION_SMOOTH: {
			Vector2 force = wish_linvel - get_position();
			if (force.length_squared() < 256.0) {
				force = -get_linear_velocity().limit_length(linear_acceleration);
				// thruster_linear = Vector2();
			} else {
				force = (force.limit_length(linear_velocity_max) - get_linear_velocity()).limit_length(linear_acceleration);
				// thruster_linear = force;
			}
			if (abs(force.x) > 1.0 || abs(force.y) > 1.0) {
				apply_central_force(force);
			}

			thruster_linear = force;

		} break;
		case LINVEL_POSITION_OVERSHOOT: {
			Vector2 force = get_position().direction_to(wish_linvel);
			if (force.length_squared() < 0.5) {
				force = Vector2(0.0, -1.0);
			}
			force = (force * linear_velocity_max - get_linear_velocity()).limit_length(linear_acceleration);
			if (abs(force.x) > 1.0 || abs(force.y) > 1.0) {
				apply_central_force(force);
			}

			thruster_linear = force;
		} break;
		case LINVEL_FORCE_ABSOLUTE: {
			wish_linvel = wish_linvel.limit_length(1.0);

			Vector2 force = (wish_linvel * linear_velocity_max - get_linear_velocity()).limit_length(linear_acceleration);
			if (abs(force.x) > 1.0 || abs(force.y) > 1.0) {
				apply_central_force(force);
			}

			thruster_linear = wish_linvel * linear_acceleration;
		} break;
		case LINVEL_FORCE_RELATIVE: {
			wish_linvel = wish_linvel.limit_length(1.0);

			Vector2 rot = wish_linvel.rotated(get_rotation());
			Vector2 force = (rot * linear_velocity_max - get_linear_velocity()).limit_length(linear_acceleration);
			if (abs(force.x) > 1.0 || abs(force.y) > 1.0) {
				apply_central_force(force);
			}

			thruster_linear = rot * linear_acceleration;
		} break;
	}

	switch (wish_angvel_type) {
		case ANGVEL_NONE: {
			thruster_angular = 0.0;
		} break;
		case ANGVEL_KEEP: {
			if (abs(get_angular_velocity()) > angular_velocity_max) {
				f32 force = CLAMP(get_angular_velocity(), -angular_velocity_max, angular_velocity_max);
				force = CLAMP(force - get_angular_velocity(), -angular_acceleration, angular_acceleration);
				apply_torque(force);
			}

			thruster_angular = 0.0;
		} break;
		case ANGVEL_CANCEL: {
			f32 force = CLAMP(-get_angular_velocity(), -angular_acceleration, angular_acceleration);
			if (abs(force) > 0.01) {
				apply_torque(force);
			}

			thruster_angular = force;
		} break;
		case ANGVEL_AIM_SMOOTH: {
			f32 offset = to_local(wish_angvel).angle();
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
		case ANGVEL_FORCE: {
			wish_angvel.x = CLAMP(wish_angvel.x, -1.0, 1.0);

			f32 force = CLAMP(wish_angvel.x * angular_velocity_max - get_angular_velocity(), -angular_acceleration, angular_acceleration);
			if (abs(force) > 0.01) {
				apply_torque(force);
			}

			thruster_angular = wish_angvel.x * angular_acceleration;
		} break;
	}
}

void HullBase::set_wish_linear_velocity_type(WishLinearVelocityType p_wish_linear_velocity_type) {
	wish_linvel_type = p_wish_linear_velocity_type;
}

WishLinearVelocityType HullBase::get_wish_linear_velocity_type() const {
	return wish_linvel_type;
}

void HullBase::set_wish_linear_velocity(Vector2 p_wish_linear_velocity) {
	wish_linvel = p_wish_linear_velocity;
}

Vector2 HullBase::get_wish_linear_velocity() const {
	return wish_linvel;
}

void HullBase::set_wish_angular_velocity_type(WishAngularVelocityType p_wish_angular_velocity_type) {
	wish_angvel_type = p_wish_angular_velocity_type;
}

WishAngularVelocityType HullBase::get_wish_angular_velocity_type() const {
	return wish_angvel_type;
}

void HullBase::set_wish_angular_velocity(Vector2 p_wish_angular_velocity) {
	wish_angvel = p_wish_angular_velocity;
}

Vector2 HullBase::get_wish_angular_velocity() const {
	return wish_angvel;
}

void HullBase::set_linear_acceleration(f32 p_linear_acceleration) {
	linear_acceleration = p_linear_acceleration;
}

f32 HullBase::get_linear_acceleration() const {
	return linear_acceleration;
}

void HullBase::set_linear_velocity_max(f32 p_linear_velocity_max) {
	linear_velocity_max = p_linear_velocity_max;
}

f32 HullBase::get_linear_velocity_max() const {
	return linear_velocity_max;
}

void HullBase::set_angular_acceleration(f32 p_angular_acceleration) {
	angular_acceleration = p_angular_acceleration;
}

f32 HullBase::get_angular_acceleration() const {
	return angular_acceleration;
}

void HullBase::set_angular_velocity_max(f32 p_angular_velocity_max) {
	angular_velocity_max = p_angular_velocity_max;
}

f32 HullBase::get_angular_velocity_max() const {
	return angular_velocity_max;
}

void HullBase::set_turrets(TypedArray<Node2D> p_turrets) {
	turrets = p_turrets;
}

TypedArray<Node2D> HullBase::get_turrets() const {
	return turrets;
}

void HullBase::set_armor_cells(PackedByteArray p_armor_cells) {
	armor_cells = p_armor_cells;
}

PackedByteArray HullBase::get_armor_cells() const {
	return armor_cells;
}

void HullBase::set_armor_cells_max(PackedByteArray p_armor_cells_max) {
	armor_cells_max = p_armor_cells_max;
}

PackedByteArray HullBase::get_armor_cells_max() const {
	return armor_cells_max;
}

void HullBase::set_armor_max(f32 p_armor_max) {
	armor_max = p_armor_max;
}

f32 HullBase::get_armor_max() const {
	return armor_max;
}

void HullBase::set_hull(f32 p_hull) {
	hull = p_hull;
}

f32 HullBase::get_hull() const {
	return hull;
}

void HullBase::set_hull_max(f32 p_hull_max) {
	hull_max = p_hull_max;
}

f32 HullBase::get_hull_max() const {
	return hull_max;
}

HullBase::HullBase() {
	set_center_of_mass_mode(CenterOfMassMode::CENTER_OF_MASS_MODE_CUSTOM);
	set_physics_process(true); // todo: may noy trigger
	// set_use_custom_integrator(true);

	// wish_linvel_type = LINVEL_NONE;
	// wish_linvel = Vector2();
	// wish_angvel_type = ANGVEL_NONE;
	// wish_angvel = Vector2();
	// linacc = 0;
	// max_linvel = 0;
	// angacc = 0;
	// max_angvel = 0;
	// armor_max = 0;
	// hull = 0;
	// hull_max = 0;
}
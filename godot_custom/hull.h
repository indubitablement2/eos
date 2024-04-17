#ifndef HULL_H
#define HULL_H

#include "core/io/resource.h"
#include "core/math/vector2.h"
#include "core/math/vector3.h"
#include "core/object/ref_counted.h"
#include "core/variant/typed_array.h"
#include "core/variant/variant.h"
#include "preludes.h"
#include "scene/2d/node_2d.h"
#include "scene/2d/physics_body_2d.h"
#include "servers/physics_server_2d.h"

#define STRINGIFY(x) #x

#define SET_GET_DEF(name, type, def) \
	type name = def;                 \
	void set_##name(type value);     \
	type get_##name() const;

#define SET_GET_BIND(name, type, class_name)                                                 \
	ClassDB::bind_method(D_METHOD(STRINGIFY(set_##name), "value"), &class_name::set_##name); \
	ClassDB::bind_method(D_METHOD(STRINGIFY(get_##name)), &class_name::get_##name);

#define SET_GET_IMPL(name, type, class_name)  \
	void class_name::set_##name(type value) { \
		name = value;                         \
	}                                         \
	type class_name::get_##name() const {     \
		return name;                          \
	}

#define PROP_DEF(name)                        \
	i32 name##_flat_increase;                 \
	i32 name##_percent_increase;              \
	f32 name##_get() const;                   \
	void name##_add_flat_increase(i32 value); \
	void name##_add_percent_increase(i32 value);

#define PROP_BIND(name)                                                                                            \
	ClassDB::bind_method(D_METHOD(STRINGIFY(name##_get)), &Hull::name##_get);                                      \
	ClassDB::bind_method(D_METHOD(STRINGIFY(name##_add_flat_increase), "value"), &Hull::name##_add_flat_increase); \
	ClassDB::bind_method(D_METHOD(STRINGIFY(name##_add_percent_increase), "value"), &Hull::name##_add_percent_increase);

#define PROP_IMPL(name)                                                                        \
	f32 Hull::name##_get() const {                                                             \
		return (data->name + f32(name##_flat_increase)) * f32(name##_percent_increase) * 0.01; \
	}                                                                                          \
	void Hull::name##_add_flat_increase(i32 value) {                                           \
		name##_flat_increase += value;                                                         \
	}                                                                                          \
	void Hull::name##_add_percent_increase(i32 value) {                                        \
		name##_percent_increase += value;                                                      \
	}

#define DATA_PROP_DEF(name, def) \
	f32 name = def;              \
	void set_##name(f32 value);  \
	f32 get_##name() const;

#define DATA_PROP_BIND(name)                                                               \
	ClassDB::bind_method(D_METHOD(STRINGIFY(set_##name), "value"), &HullData::set_##name); \
	ClassDB::bind_method(D_METHOD(STRINGIFY(get_##name)), &HullData::get_##name);          \
	ADD_PROPERTY(PropertyInfo(Variant::FLOAT, #name), STRINGIFY(set_##name), STRINGIFY(get_##name));

#define DATA_PROP_IMPL(name)               \
	void HullData::set_##name(f32 value) { \
		name = value;                      \
	}                                      \
	f32 HullData::get_##name() const {     \
		return name;                       \
	}

enum WishLinearVelocityType {
	WISH_LINEAR_VELOCITY_TYPE_NONE,
	WISH_LINEAR_VELOCITY_TYPE_KEEP,
	WISH_LINEAR_VELOCITY_TYPE_CANCEL,
	WISH_LINEAR_VELOCITY_TYPE_POSITION_SMOOTH,
	WISH_LINEAR_VELOCITY_TYPE_POSITION_OVERSHOOT,
	WISH_LINEAR_VELOCITY_TYPE_FORCE_ABSOLUTE,
	WISH_LINEAR_VELOCITY_TYPE_FORCE_RELATIVE
};

enum WishAngularVelocityType {
	WISH_ANGULAR_VELOCITY_TYPE_NONE,
	WISH_ANGULAR_VELOCITY_TYPE_KEEP,
	WISH_ANGULAR_VELOCITY_TYPE_CANCEL,
	WISH_ANGULAR_VELOCITY_TYPE_AIM_SMOOTH,
	WISH_ANGULAR_VELOCITY_TYPE_FORCE
};

class HullData : public Resource {
	GDCLASS(HullData, Resource);

protected:
	static void _bind_methods();

public:
	SET_GET_DEF(armor_cells_max, PackedByteArray, PackedByteArray());

	DATA_PROP_DEF(linear_acceleration, 100.0);
	DATA_PROP_DEF(linear_velocity_max, 100.0);
	DATA_PROP_DEF(angular_acceleration, 10.0);
	DATA_PROP_DEF(angular_velocity_max, 10.0);
	DATA_PROP_DEF(armor_max, 0.0);
	DATA_PROP_DEF(hull_max, 100.0);
};

class Hull : public RigidBody2D {
	GDCLASS(Hull, RigidBody2D);

protected:
	static void _bind_methods();
	void _notification(int p_what);

private:
	void _apply_wish_movement();

public:
	Vector2 thruster_linear;
	f32 thruster_angular; // 4

	SET_GET_DEF(data, Ref<HullData>, Ref<HullData>());
	SET_GET_DEF(wish_linear_velocity_type, WishLinearVelocityType, WISH_LINEAR_VELOCITY_TYPE_NONE);
	SET_GET_DEF(wish_linear_velocity, Vector2, Vector2());
	SET_GET_DEF(wish_angular_velocity_type, WishAngularVelocityType, WISH_ANGULAR_VELOCITY_TYPE_NONE);
	SET_GET_DEF(wish_angular_velocity, Vector2, Vector2());
	SET_GET_DEF(turrets, TypedArray<Node2D>, TypedArray<Node2D>());
	SET_GET_DEF(armor_cells, PackedByteArray, PackedByteArray());
	SET_GET_DEF(hull_relative, f32, 1.0);

	PROP_DEF(linear_acceleration);
	PROP_DEF(linear_velocity_max);
	PROP_DEF(angular_acceleration);
	PROP_DEF(angular_velocity_max);
	PROP_DEF(armor_max);
	PROP_DEF(hull_max);

	// f32 get_total_armor_damage() const;
	// Vector3 repair_damaged_armor_cell(f32 max_repair);

	// // 1+2x armor cell changes from full
	// PackedByteArray get_first_state() const;
	// // 4-6 position
	// // 2 rotation
	// // 1+2x armor changes
	// // 1x turrets rotation
	// // 1 acceleration bitfield
	// // 2-3 hull
	// PackedByteArray get_delta_state() const;

	Hull();
};

VARIANT_ENUM_CAST(WishLinearVelocityType);
VARIANT_ENUM_CAST(WishAngularVelocityType);

#endif
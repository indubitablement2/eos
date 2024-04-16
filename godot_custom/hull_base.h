#ifndef HULL_BASE_H
#define HULL_BASE_H

#include "core/math/vector2.h"
#include "core/math/vector3.h"
#include "core/variant/typed_array.h"
#include "core/variant/variant.h"
#include "preludes.h"
#include "scene/2d/node_2d.h"
#include "scene/2d/physics_body_2d.h"
#include "servers/physics_server_2d.h"

// GLOBAL_DEF(PropertyInfo(Variant::INT, "editor/naming/node_name_num_separator", PROPERTY_HINT_ENUM, "None,Space,Underscore,Dash"), 0);

enum WishLinearVelocityType {
	LINVEL_NONE,
	LINVEL_KEEP,
	LINVEL_CANCEL,
	LINVEL_POSITION_SMOOTH,
	LINVEL_POSITION_OVERSHOOT,
	LINVEL_FORCE_ABSOLUTE,
	LINVEL_FORCE_RELATIVE
};

enum WishAngularVelocityType {
	ANGVEL_NONE,
	ANGVEL_KEEP,
	ANGVEL_CANCEL,
	ANGVEL_AIM_SMOOTH,
	ANGVEL_FORCE
};

class HullBase : public RigidBody2D {
	GDCLASS(HullBase, RigidBody2D);

protected:
	static void _bind_methods();
	void _notification(int p_what);

private:
	void _apply_wish_movement();

public:
	WishLinearVelocityType wish_linvel_type;
	Vector2 wish_linvel;

	WishAngularVelocityType wish_angvel_type;
	Vector2 wish_angvel;

	f32 linear_acceleration;
	f32 linear_velocity_max;
	f32 angular_acceleration;
	f32 angular_velocity_max;

	Vector2 thruster_linear;
	f32 thruster_angular; // 4

	TypedArray<Node2D> turrets;

	PackedByteArray armor_cells;
	PackedByteArray armor_cells_max;
	f32 armor_max;
	// f32

	f32 hull;
	f32 hull_max;

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

	void set_wish_linear_velocity_type(WishLinearVelocityType p_wish_linear_velocity_type);
	WishLinearVelocityType get_wish_linear_velocity_type() const;
	void set_wish_linear_velocity(Vector2 p_wish_linear_velocity);
	Vector2 get_wish_linear_velocity() const;

	void set_wish_angular_velocity_type(WishAngularVelocityType p_wish_angular_velocity_type);
	WishAngularVelocityType get_wish_angular_velocity_type() const;
	void set_wish_angular_velocity(Vector2 p_wish_angular_velocity);
	Vector2 get_wish_angular_velocity() const;

	void set_linear_acceleration(f32 p_linear_acceleration);
	f32 get_linear_acceleration() const;
	void set_linear_velocity_max(f32 p_linear_velocity_max);
	f32 get_linear_velocity_max() const;
	void set_angular_acceleration(f32 p_angular_acceleration);
	f32 get_angular_acceleration() const;
	void set_angular_velocity_max(f32 p_angular_velocity_max);
	f32 get_angular_velocity_max() const;

	void set_turrets(TypedArray<Node2D> p_turrets);
	TypedArray<Node2D> get_turrets() const;

	void set_armor_cells(PackedByteArray p_armor_cells);
	PackedByteArray get_armor_cells() const;
	void set_armor_cells_max(PackedByteArray p_armor_cells_max);
	PackedByteArray get_armor_cells_max() const;
	void set_armor_max(f32 p_armor_max);
	f32 get_armor_max() const;

	void set_hull(f32 p_hull);
	f32 get_hull() const;
	void set_hull_max(f32 p_hull_max);
	f32 get_hull_max() const;

	HullBase();
};

VARIANT_ENUM_CAST(WishLinearVelocityType);
VARIANT_ENUM_CAST(WishAngularVelocityType);

#endif
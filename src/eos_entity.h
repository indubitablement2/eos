#ifndef EOS_ENTITY
#define EOS_ENTITY

#include "godot_cpp/classes/physics_direct_body_state2d.hpp"
#include "godot_cpp/classes/resource.hpp"
#include "godot_cpp/variant/string.hpp"
#include "godot_cpp/variant/vector2.hpp"
#include <cstdint>
#include <godot_cpp/classes/rigid_body2d.hpp>

using namespace godot;

class EosEntity : public RigidBody2D {
	GDCLASS(EosEntity, RigidBody2D);

private:
	enum WishLinearVelocity {
		LINVEL_KEEP,
		LINVEL_CANCEL,
		LINVEL_POSITION,
		LINVEL_POSITION_OVERSHOT,
		LINVEL_ABSOLUTE,
		LINVEL_RELATIVE,
	};

	WishLinearVelocity wish_linear_velocity_type = LINVEL_KEEP;
	Vector2 wish_linear_velocity = Vector2(0, 0);

	enum WishAngularVelocity {
		ANGVEL_KEEP,
		ANGVEL_CANCEL,
		ANGVEL_AIM,
		ANGVEL_FORCE,
	};
	WishAngularVelocity wish_angular_velocity_type = ANGVEL_KEEP;
	Vector2 wish_angular_velocity = Vector2(0, 0);

protected:
	static void _bind_methods();

	void compute_stats();

public:
	float base_linear_acceleration;
	float base_angular_acceleration;
	float base_max_linear_velocity;
	float base_max_angular_velocity;

	void set_base_linear_acceleration(double value);
	double get_base_linear_acceleration() const;
	void set_base_angular_acceleration(double value);
	double get_base_angular_acceleration() const;
	void set_base_max_linear_velocity(double value);
	double get_base_max_linear_velocity() const;
	void set_base_max_angular_velocity(double value);
	double get_base_max_angular_velocity() const;

	float linear_acceleration;
	float angular_acceleration;
	float max_linear_velocity;
	float max_angular_velocity;

	float base_readiness = 1.0f;
	void set_base_readiness(double value);
	double get_base_readiness() const;

	float readiness;
	float hull_hp;
	// TODO: Replace with armor grid
	float armor_hp;

	void set_readiness(double value);
	double get_readiness() const;
	void set_hull_hp(double value);
	double get_hull_hp() const;
	void set_average_armor_hp(double value);
	double get_average_armor_hp() const;

	// Keep current linear velocity. eg. do nothing.
	void set_wish_linear_velocity_keep();
	// Try to reach 0 linear velocity.
	void set_wish_linear_velocity_cancel();
	// Cancel our current velocity to reach position as fast as possible.
	// Does not overshot.
	void set_wish_linear_velocity_position(Vector2 position);
	// Same as position, but always try to go at max velocity.
	void set_wish_linear_velocity_position_overshot(Vector2 position);
	// Force toward an absolute direction. -y is up.
	// Magnitude bellow 1 can be used to accelerate slower.
	// Magnitude clamped to 1.
	void set_wish_linear_velocity_absolute(Vector2 direction);
	// Force toward a direction relative to current rotation. +y is forward.
	// Magnitude bellow 1 can be used to accelerate slower.
	// Magnitude clamped to 1.
	void set_wish_linear_velocity_relative(Vector2 direction);

	// Keep current angular velocity. eg. do nothing.
	void set_wish_angular_velocity_keep();
	// Try to reach 0 angular velocity.
	void set_wish_angular_velocity_cancel();
	// Set angular velocity to face world space position without overshot.
	void set_wish_angular_velocity_aim(Vector2 position);
	// Rotate left or right [-1..1].
	// Force will be clamped.
	void set_wish_angular_velocity_force(double force);

	void physics_process();
	void integrate_forces(PhysicsDirectBodyState2D *state);
};

#endif
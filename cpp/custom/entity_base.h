#ifndef ENTITY_BASE
#define ENTITY_BASE

#include "preludes.h"
#include "scene/2d/physics_body_2d.h"

class EntityBase : public RigidBody2D {
	GDCLASS(EntityBase, RigidBody2D);

private:
	enum WishLinvel {
		LINVEL_KEEP,
		LINVEL_STOP,
		LINVEL_POSITION,
		LINVEL_POSITION_OVERSHOOT,
		LINVEL_ABSOLUTE,
		LINVEL_RELATIVE,
	};

	WishLinvel wish_linvel_type = LINVEL_KEEP;
	Vector2 wish_linvel = Vector2(0, 0);

	enum WishAngvel {
		ANGVEL_KEEP,
		ANGVEL_STOP,
		ANGVEL_AIM,
		ANGVEL_FORCE,
	};
	WishAngvel wish_angvel_type = ANGVEL_KEEP;
	Vector2 wish_angvel = Vector2(0, 0);

protected:
	static void _bind_methods();

	void compute_stats();

public:
	ADD_SETGET_MODIFIERS(f32, linacc, 800.0f)
	ADD_SETGET_MODIFIERS(f32, max_linvel, 400.0f)
	ADD_SETGET_MODIFIERS(f32, angacc, 8.0f)
	ADD_SETGET_MODIFIERS(f32, max_angvel, 4.0f)

	ADD_SETGET_MODIFIERS(f32, max_hull_hp, 100.0f)
	ADD_SETGET_MODIFIERS(f32, max_armor_hp, 0.0f)

	// Keep current linear velocity. eg. do nothing.
	void wish_linvel_keep();
	// Try to reach 0 linear velocity.
	void wish_linvel_stop();
	// Cancel our current velocity to reach position as fast as possible.
	// Does not overshot.
	void wish_linvel_position(Vector2 position);
	// Same as position, but always try to go at max velocity.
	void wish_linvel_position_overshoot(Vector2 position);
	// Force toward an absolute direction. -y is up.
	// Magnitude bellow 1 can be used to accelerate slower.
	// Magnitude clamped to 1.
	void wish_linvel_absolute(Vector2 direction);
	// Force toward a direction relative to current rotation. +y is forward.
	// Magnitude bellow 1 can be used to accelerate slower.
	// Magnitude clamped to 1.
	void wish_linvel_relative(Vector2 direction);

	// Keep current angular velocity. eg. do nothing.
	void wish_angvel_keep();
	// Try to reach 0 angular velocity.
	void wish_angvel_stop();
	// Set angular velocity to face world space position without overshoot.
	void wish_angvel_aim(Vector2 position);
	// Set angular velocity to reach this rotation.
	void wish_angvel_rotation(f32 rotation);
	// Rotate left or right [-1..1].
	// Force will be clamped.
	void wish_angvel_force(f32 force);

	void _base_integrate_forces(PhysicsDirectBodyState2D *state);
};

#endif
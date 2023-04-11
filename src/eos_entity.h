#ifndef EOS_ENTITY
#define EOS_ENTITY

#include "godot_cpp/variant/vector2.hpp"
#include <cstdint>
#include <godot_cpp/classes/rigid_body2d.hpp>

using namespace godot;

class EosEntity : public RigidBody2D {
  GDCLASS(EosEntity, RigidBody2D);

private:
  enum WishLinVel {
    // Keep current velocity. eg. do nothing.
    LINVEL_KEEP,
    // Try to reach 0 linvel.
    LINVEL_CANCEL,
    // Cancel our current velocity to reach position as fast as possible.
    // Does not overshot.
    LINVEL_POSITION,
    // Same as position, but always try to go at max velocity.
    LINVEL_POSITION_OVERSHOT,
    // `-y` is up.
    // Magnitude 0 to 1
    LINVEL_ABSOLUTE,
    // Relative to current rotation.
    // `+y` is forward.
    // Magnitude 0 to 1
    LINVEL_RELATIVE,
  };

  WishLinVel wish_linvel_type = LINVEL_KEEP;
  Vector2 wish_linvel = Vector2(0, 0);

  enum WishAngVel {
    // Keep current angvel. eg. do nothing.
    ANGVEL_KEEP,
    // Try to reach 0 angvel.
    ANGVEL_CANCEL,
    // Set angvel to face world space position without overshot.
    ANGVEL_AIM,
    // Set angvel to reach an obsolute rotation without overshot.
    ANGVEL_ROTATION,
    // Rotate left or right [-1..1].
    ANGVEL_FORCE,
  };
  WishAngVel wish_angvel_type = ANGVEL_KEEP;
  Vector2 wish_angvel = Vector2(0, 0);

protected:
  static void _bind_methods();

public:
  // If this entity is a ship from a fleet.
  int64_t fleet_id = -1;
  int ship_idx = -1;

  // Owner client id. -1 if no owner.
  int64_t owner = -1;
  int64_t team = -1;

  float linear_acceleration = 0.0;
  float angular_acceleration = 0.0;
  float max_linear_velocity = 0.0;
  float max_angular_velocity = 0.0;

  float readiness = 0.0;
  int hull_hp = 0;
  int armor_hp = 0;

  void set_wish_linvel_keep();
  void set_wish_linvel_cancel();
  void set_wish_linvel_position(Vector2 position);
  void set_wish_linvel_position_overshot(Vector2 position);
  void set_wish_linvel_absolute(Vector2 direction);
  void set_wish_linvel_relative(Vector2 direction);

  void set_wish_angvel_keep();
  void set_wish_angvel_cancel();
  void set_wish_angvel_aim(Vector2 position);
  void set_wish_angvel_rotation(float rotation);
  void set_wish_angvel_force(float force);

  void _integrate_forces(PhysicsDirectBodyState2D *state) override;
};

#endif
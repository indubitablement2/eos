#ifndef ENEMY_H
#define ENEMY_H

#include "core/config/engine.h"
#include "core/math/vector2.h"
#include "preludes.h"
#include "scene/2d/path_2d.h"

class Enemy : public PathFollow2D {
	GDCLASS(Enemy, PathFollow2D);

protected:
	void _notification(int p_what);
	static void _bind_methods();

public:
	f32 speed = 100.0f;
	f32 radius = 64.0f;
	f32 hp = 100.0f;

	f32 get_speed() const { return speed; }
	void set_speed(f32 value) { speed = value; }

	f32 get_radius() const { return radius; }
	void set_radius(f32 value) {
		radius = value;
		if (Engine::get_singleton()->is_editor_hint()) {
			queue_redraw();
		}
	}

	f32 get_hp() const { return hp; }
	void set_hp(f32 value) { hp = value; }

	bool hit(f32 damage) {
		hp -= damage;

		if (hp <= 0.0f) {
			queue_free();
			return true;
		} else {
			return false;
		}
	}
};

#endif
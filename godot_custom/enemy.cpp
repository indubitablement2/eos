#include "enemy.h"
#include "core/string/print_string.h"
#include "grid.h"
#include "scene/main/canvas_item.h"
#include "scene/main/node.h"

void Enemy::_bind_methods() {
	ClassDB::bind_method(D_METHOD("set_speed", "speed"), &Enemy::set_speed);
	ClassDB::bind_method(D_METHOD("get_speed"), &Enemy::get_speed);
	ADD_PROPERTY(PropertyInfo(Variant::FLOAT, "speed"), "set_speed", "get_speed");

	ClassDB::bind_method(D_METHOD("set_radius", "radius"), &Enemy::set_radius);
	ClassDB::bind_method(D_METHOD("get_radius"), &Enemy::get_radius);
	ADD_PROPERTY(PropertyInfo(Variant::FLOAT, "radius"), "set_radius", "get_radius");

	ClassDB::bind_method(D_METHOD("set_hp", "hp"), &Enemy::set_hp);
	ClassDB::bind_method(D_METHOD("get_hp"), &Enemy::get_hp);
	ADD_PROPERTY(PropertyInfo(Variant::FLOAT, "hp"), "set_hp", "get_hp");

	ClassDB::bind_method(D_METHOD("hit", "damage"), &Enemy::hit);
}

void Enemy::_notification(int p_what) {
	switch (p_what) {
		case NOTIFICATION_READY: {
			if (!Engine::get_singleton()->is_editor_hint()) {
				set_physics_process(true);
			}
			break;
		}
		case NOTIFICATION_DRAW: {
			if (Engine::get_singleton()->is_editor_hint()) {
				draw_circle(Vector2(), radius, Color(1.0f, 0.0f, 0.0f, 0.5f));
			}
			break;
		}
		case NOTIFICATION_PHYSICS_PROCESS: {
			set_progress(get_progress() + speed * get_physics_process_delta_time());
			Grid::add_enemy(this);

			break;
		}
	}
}
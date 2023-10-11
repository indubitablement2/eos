#include "entity.h"
#include "entt/src/entt/entity/fwd.hpp"
#include "preludes.h"

ADD_SETGET_IMPL(Entity, bool, am_i_test)

void Entity::_bind_methods() {
	ADD_SETGET_PROPERTY(Entity, Variant::BOOL, am_i_test)

	// ClassDB::bind_method(
	// 		D_METHOD("test_me", "name", "value"),
	// 		&Entity::test_me);
}

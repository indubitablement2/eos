#ifndef ENTITY
#define ENTITY

#include "preludes.h"
#include "scene/2d/node_2d.h"

class Entity : public Node2D {
	GDCLASS(Entity, Node2D);

protected:
	static void _bind_methods();

public:
	ADD_SETGET(bool, am_i_test, false)
};

#endif
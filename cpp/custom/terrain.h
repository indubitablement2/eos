#ifndef TERRAIN
#define TERRAIN

#include "core/io/resource.h"
#include "core/math/vector2i.h"
#include "preludes.h"

class Terrain : public Resource {
	GDCLASS(Terrain, Resource);

protected:
	static void _bind_methods();

public:
	ADD_SETGET(bool, am_i_test, false)

	GDVIRTUAL2(_cb_pawn_enter, Vector2i, bool)
	GDVIRTUAL2(_cb_pawn_exit, Vector2i, bool)
};

#endif
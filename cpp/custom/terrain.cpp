#include "terrain.h"
#include "preludes.h"

ADD_SETGET_IMPL(Terrain, bool, am_i_test)

void Terrain::_bind_methods() {
	ADD_SETGET_PROPERTY(Terrain, Variant::BOOL, am_i_test)

	// ClassDB::bind_method(
	// 		D_METHOD("test_me", "name", "value"),
	// 		&Terrain::test_me);

	GDVIRTUAL_BIND(_cb_pawn_enter, "coordinates", "pawn");
	GDVIRTUAL_BIND(_cb_pawn_exit, "coordinates", "pawn");

	GDVIRTUAL_CALL(_cb_pawn_enter, Vector2i(0, 0), true);
}

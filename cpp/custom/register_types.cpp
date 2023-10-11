#include "register_types.h"
#include "core/object/class_db.h"

#include "entity.h"
#include "grid.h"
#include "pawn.h"

// pawn
// animal

// item

// furniture/walls
// floor
// terrain

void initialize_custom_module(ModuleInitializationLevel p_level) {
	if (p_level != MODULE_INITIALIZATION_LEVEL_SCENE) {
		return;
	}

	ClassDB::register_class<Pawn>();
	ClassDB::register_class<Grid>();
	ClassDB::register_class<Entity>();
}

void uninitialize_custom_module(ModuleInitializationLevel p_level) {
	if (p_level != MODULE_INITIALIZATION_LEVEL_SCENE) {
		return;
	}
}

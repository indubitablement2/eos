#include "register_types.h"
#include "core/object/class_db.h"
#include "grid.h"

void initialize_custom_module(ModuleInitializationLevel p_level) {
	if (p_level != MODULE_INITIALIZATION_LEVEL_SCENE) {
		return;
	}

	ClassDB::register_class<Enemy>();
	ClassDB::register_abstract_class<Grid>();
}

void uninitialize_custom_module(ModuleInitializationLevel p_level) {
	if (p_level != MODULE_INITIALIZATION_LEVEL_SCENE) {
		return;
	}
}

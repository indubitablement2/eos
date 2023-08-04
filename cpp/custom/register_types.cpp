#include "register_types.h"
#include "core/object/class_db.h"

#include "entity_base.h"

void initialize_custom_module(ModuleInitializationLevel p_level) {
	if (p_level != MODULE_INITIALIZATION_LEVEL_SCENE) {
		return;
	}

	ClassDB::register_class<EntityBase>();
}

void uninitialize_custom_module(ModuleInitializationLevel p_level) {
	if (p_level != MODULE_INITIALIZATION_LEVEL_SCENE) {
		return;
	}
}

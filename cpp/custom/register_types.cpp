#include "register_types.h"
#include "core/object/class_db.h"

#include "entity_base.h"

void initialize_custom_module(ModuleInitializationLevel p_level) {
	if (p_level != MODULE_INITIALIZATION_LEVEL_SCENE) {
		return;
	}

	ClassDB::register_class<EntityBase>();
	//ClassDB::register_abstract_class<Generation>();

	//ClassDB::register_class<GridCharacterBody>();
	//ClassDB::register_class<GridBiomeScanner>();
}

void uninitialize_custom_module(ModuleInitializationLevel p_level) {
	if (p_level != MODULE_INITIALIZATION_LEVEL_SCENE) {
		return;
	}
}

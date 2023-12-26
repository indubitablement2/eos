#include "register_types.h"
#include "codec.h"
#include "core/object/class_db.h"

void initialize_godot_custom_module(ModuleInitializationLevel p_level) {
	if (p_level != MODULE_INITIALIZATION_LEVEL_SCENE) {
		return;
	}

	ClassDB::register_class<ClientCodec>();
}

void uninitialize_godot_custom_module(ModuleInitializationLevel p_level) {
	if (p_level != MODULE_INITIALIZATION_LEVEL_SCENE) {
		return;
	}
}

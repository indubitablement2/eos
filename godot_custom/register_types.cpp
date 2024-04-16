#include "register_types.h"
#include "core/object/class_db.h"
#include "hull_base.h"
#include "postcard_codec.h"

void initialize_godot_custom_module(ModuleInitializationLevel p_level) {
	if (p_level != MODULE_INITIALIZATION_LEVEL_SCENE) {
		return;
	}

	ClassDB::register_abstract_class<PostcardCodec>();
	ClassDB::register_class<HullBase>();
}

void uninitialize_godot_custom_module(ModuleInitializationLevel p_level) {
	if (p_level != MODULE_INITIALIZATION_LEVEL_SCENE) {
		return;
	}
}

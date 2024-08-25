extends Resource
class_name Scenario

enum ScenarioDifficultyString {
	UNKNOWN,
	EASY,
	NORMAL,
	BANANA,
}
@export var difficulty := ScenarioDifficultyString.UNKNOWN

@export var battle_scene: PackedScene


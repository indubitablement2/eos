extends Node
# Manage volume.

const MASTER = "Master"
const MUSIC = "Music"
const SFXUI = "SfxUi"
const SFX2D = "Sfx2d"


func _ready() -> void:
	# master_volume
	AudioServer.set_bus_volume_db(AudioServer.get_bus_index(MASTER), linear2db(ProjectSettings.get_setting(GlobalVariable.MASTER_VOLUME)))
	# music_volume
	AudioServer.set_bus_volume_db(AudioServer.get_bus_index(MUSIC), linear2db(ProjectSettings.get_setting(GlobalVariable.MUSIC_VOLUME)))
	# sfxui_volume
	AudioServer.set_bus_volume_db(AudioServer.get_bus_index(SFXUI), linear2db(ProjectSettings.get_setting(GlobalVariable.SFXUI_VOLUME)))
	# sfx2d_volume
	AudioServer.set_bus_volume_db(AudioServer.get_bus_index(SFX2D), linear2db(ProjectSettings.get_setting(GlobalVariable.SFX2D_VOLUME)))


func set_master_volume(volume := 1.0) -> void:
	# Apply change.
	AudioServer.set_bus_volume_db(AudioServer.get_bus_index(MASTER), linear2db(volume))
	# Save.
	ProjectSettings.set_setting(GlobalVariable.MASTER_VOLUME, volume)


func set_music_volume(volume:float = 1.0) -> void:
	# Apply change.
	AudioServer.set_bus_volume_db(AudioServer.get_bus_index(MUSIC), linear2db(volume))
	# Save.
	ProjectSettings.set_setting(GlobalVariable.MUSIC_VOLUME, volume)


func set_sfxui_volume(volume:float = 1.0) -> void:
	# Apply change.
	AudioServer.set_bus_volume_db(AudioServer.get_bus_index(SFXUI), linear2db(volume))
	# Save.
	ProjectSettings.set_setting(GlobalVariable.SFXUI_VOLUME, volume)


func set_sfx2d_volume(volume:float = 1.0) -> void:
	# Apply change.
	AudioServer.set_bus_volume_db(AudioServer.get_bus_index(SFX2D), linear2db(volume))
	# Save.
	ProjectSettings.set_setting(GlobalVariable.SFX2D_VOLUME, volume)

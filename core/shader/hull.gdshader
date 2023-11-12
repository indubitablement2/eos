shader_type canvas_item;

//#define EXTRA_SCALE 8.0

uniform sampler2D armor_max_relative_texture;
uniform sampler2D armor_relative_texture;
uniform sampler2D recent_damage_texture;

void fragment() {
	vec4 tex = texture(TEXTURE, UV);
	float armor_max = texture(armor_max_relative_texture, UV).r;
	float armor = texture(armor_relative_texture, UV).r;
	float recent_damage = texture(recent_damage_texture, UV).r;
	
	vec4 col = tex;
	
	float armor_left = armor / armor_max;
	col.rgb *= armor_left * 0.5 + 0.5;
	
	float value = (tex.r + tex.g + tex.b) * 0.4;
	col.r += recent_damage * value;
	
	NORMAL_MAP = texture(NORMAL_TEXTURE, UV).rgb;
//	col.rgb = vec3(armor);
	COLOR = col;
}
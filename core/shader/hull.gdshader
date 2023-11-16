shader_type canvas_item;

uniform sampler2D armor_max_relative_texture;
uniform sampler2D armor_relative_texture;
uniform sampler2D recent_damage_texture;

void fragment() {
	vec4 tex = texture(TEXTURE, UV);
	vec4 col = tex;
	
	float armor_max = texture(armor_max_relative_texture, UV).r;
	float armor = texture(armor_relative_texture, UV).r;
	float armor_left = armor / armor_max;
	col.rgb *= armor_left * 0.5 + 0.5;
	
	float recent_damage = texture(recent_damage_texture, UV).r;
	// Only show ref effect when armor is low.
	recent_damage *= armor_left; 
	float value = (tex.r + tex.g + tex.b) * 0.4;
	col.r += recent_damage * value;
	
	NORMAL_MAP = texture(NORMAL_TEXTURE, UV).rgb;
//	col.rgb = vec3(armor);
	COLOR = col;
}
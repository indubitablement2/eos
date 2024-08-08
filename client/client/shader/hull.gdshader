shader_type canvas_item;

uniform sampler2D armor_max;
uniform vec2 armor_offset;

void fragment() {
	vec4 col = texture(TEXTURE, UV);
	
	vec2 center_uv = UV - 0.5;
	//col.rgb -= 10.0 * sqrt(center_uv.x * center_uv.x + center_uv.y * center_uv.y);
	
	ivec2 armor_coord = (ivec2(floor(center_uv / TEXTURE_PIXEL_SIZE / 16.0 + armor_offset / 16.0 + 0.5)));
	
	float checker_board = float((armor_coord.x + armor_coord.y) % 2);
	
	//col.rgb *= checker_board;
	
	ivec2 armor_texel = armor_coord + ivec2(4, 5);
	col.rgb *= texelFetch(armor_max, armor_texel, 0).rgb;
	
	COLOR = col;
}

shader_type canvas_item;

const uint MATERIAL_IDX_MASK = uint(1 << 12) - 1u;


void fragment() {
	ivec2 coords = ivec2((UV * vec2(textureSize(TEXTURE, 0))));
	uint data = floatBitsToUint(texelFetch(TEXTURE, coords, 0).x);
	
	uint material_idx = data & MATERIAL_IDX_MASK;
	
	float r = float((material_idx * 3u + 1u) % 11u) / 11.0;
	float g = float((material_idx * 5u + 3u) % 13u) / 13.0;
	float b = float((material_idx * 7u + 5u) % 17u) / 17.0;
	
	vec4 col = vec4(r, g, b, 1.0);
	
	COLOR = col;
}

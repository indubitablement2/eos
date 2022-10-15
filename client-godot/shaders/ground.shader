shader_type canvas_item;

// TEXTURE is terrain_data as bytes.

// 8x8 atlast of 512x512 terrain texture.
uniform sampler2D terrain_sampler;

void fragment() {
	// I made a silly mistake. 255 is the max, not 256.
	float id = texture(TEXTURE, UV).r * 255.0;
	
	// Position from (0.0, 0.0) to (1.0, 1.0). Mapped to terrain_data.
	vec2 texture_to_sample = vec2(mod(id, 8.0) * 0.125, floor(id / 8.0) * 0.125);
	
	// Sample the terrain_sampler.
	// min = 0.0, max = 0.124 to not overflow.
	vec4 c = texture(terrain_sampler, texture_to_sample + mod(UV / TEXTURE_PIXEL_SIZE / 128.0, vec2(0.125)));
	
	// Uncomment to get debug view.
//	c = vec4(texture_to_sample.x, texture_to_sample.y, 0.0, 1.0);
	
	COLOR = c;
}
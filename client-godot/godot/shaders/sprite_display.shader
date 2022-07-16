shader_type canvas_item;

//const float atlas_texture_size = 512.0;
uniform sampler2D def_texture;
uniform vec2 atlas_size = vec2(192.0, 128.0);

void vertex() {
	int def_id = floatBitsToInt(INSTANCE_CUSTOM.z);
	ivec2 def_texture_size = textureSize(def_texture, 0);
	
	vec4 def_data = texelFetch(def_texture, ivec2(def_id % def_texture_size.x, def_id / def_texture_size.y), 0);
	
	// Scale vertex.
	// Default: Goes from -0.5 to 0.5 for both x and y.
	vec2 sprite_size = (def_data.zw - def_data.xy) * atlas_size;
	VERTEX *= sprite_size;
	
	// Make UV the right size. 
	// Default: Goes from 0 to 1 on the whole texture.
	vec2 relative_scale = def_data.zw - def_data.xy;
	UV *= relative_scale;
	
	// Move UV to the right place.
	// Default: Goes from 0 to relative_scale.
	vec2 relative_position = def_data.xy;
	UV += relative_position;
}

void fragment() {
//	COLOR = COLOR * vec4(1.0, 0.5, 0.5, 1.0);
}
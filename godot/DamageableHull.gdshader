shader_type canvas_item;

// TODO: use instance uniform instead of local shader when implemented 
uniform sampler2D damage;

void fragment() {
	// TODO: modulate texture
	// TODO: damage shader
	COLOR = texture(TEXTURE, UV);
}

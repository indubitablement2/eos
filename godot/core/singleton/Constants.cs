using Godot;

public partial class Constants : Node2D
{
    public const float Delta = 1.0f / 60.0f;

    public static Texture2D ErrorTexture = GD.Load<Texture2D>("res://core/texture/util/error.png");
}
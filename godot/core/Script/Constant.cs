using Godot;

public partial class Constant : Node
{
    public static Constant Instance { get; private set; }

    public const float HalfPi = Mathf.Pi * 0.5f;

    public static StringName InputLeft = new StringName("left");
    public static StringName InputRight = new StringName("right");
    public static StringName InputUp = new StringName("up");
    public static StringName InputDown = new StringName("down");
    public static StringName InputPrimary = new StringName("primary");
    public static StringName InputSecondary = new StringName("secondary");
    public static StringName InputAimAtCursor = new StringName("aim_at_cursor");

    // static Texture2D _pixelTexture;
    // public static Texture2D PixelTexture
    // {
    //     get
    //     {
    //         _pixelTexture ??= GD.Load<Texture2D>("res://Core/Texture/Pixel.png");
    //         return _pixelTexture;
    //     }
    //     set { }
    // }
    public static Texture2D PixelTexture = GD.Load<Texture2D>("res://Core/Texture/Pixel.png");

    public override void _Ready()
    {
        Instance = this;

        // PixelTexture = GD.Load<Texture2D>("res://Core/Texture/Pixel.png");
    }
}
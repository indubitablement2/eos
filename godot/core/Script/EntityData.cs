using Godot;
using System;

[GlobalClass, Tool]
public partial class EntityData : Resource
{
    [Export]
    public Texture2D Sprite;
    [Export]
    public Vector2 SpriteOffset = Vector2.Zero;

    [Export(PropertyHint.Range, "0, 2000, 1, or_greater")]
    public float LinearAcceleration = 800.0f;
    [Export(PropertyHint.Range, "0, 2000, 1, or_greater")]
    public float LinearVelocityMax = 400.0f;
    [Export(PropertyHint.Range, "0, 100, 1, or_greater")]
    public float AngularAcceleration = 8.0f;
    [Export(PropertyHint.Range, "0, 100, 1, or_greater")]
    public float AngularVelocityMax = 4.0f;

    [Export(PropertyHint.Range, "0.01, 4, 0.01, or_greater")]
    public float LocalTimeScale = 1.0f;

    [Export]
    public float HullMax = 100.0f;

    [Export]
    public float ArmorMax = 100.0f;
    [Export]
    public bool SimpleArmor = false;

    [Export(PropertyHint.Range, "0, 1, 0.05")]
    public float ArmorMinEffectiveness = 0.1f;

    [Export]
    public Texture2D ArmorMaxRelativeTexture;

    [ExportGroup("Computed")]
    [Export]
    public Image ArmorMaxRelative;
    [Export]
    public Vector2I ArmorSize;


    public void Verify()
    {
        Sprite ??= Constant.PixelTexture;

        ArmorMaxRelativeTexture ??= Constant.PixelTexture;

        ArmorSize = (Vector2I)(
            Sprite.GetSize() / Entity.ARMOR_SCALE)
            .Ceil()
            .Clamp(Entity.ARMOR_SIZE_MIN, new Vector2(128, 128));

        ArmorMaxRelative = ArmorMaxRelativeTexture.GetImage();
        ArmorMaxRelative.Convert(Image.Format.Rf);
        ArmorMaxRelative.Resize(ArmorSize.X, ArmorSize.Y);
    }
}

using Godot;

[Tool, GlobalClass]
public partial class TurretData : Resource
{
    [Export(PropertyHint.Range, "0.0, 30.0, 0.1, or_greater")]
    public float RotationSpeed = 8.0f;

    [Export(PropertyHint.Range, "0.0, 1.0, 0.01, or_greater")]
    public float FireDelay = 0.1f;

    [ExportCategory("Ammo")]

    [Export]
    public int AmmoMax = 1000000;
    /// <summary>
    /// How long to refill one ammo.
    /// </summary>
    [Export]
    public float AmmoReplenishDelay = float.PositiveInfinity;
    [Export]
    public int AmmoReplenishAmount = 1;

    [ExportCategory("Ai")]

    [Export(PropertyHint.Range, "0.0, 2000.0, 1.0, or_greater")]
    public float EffectiveRange = 500.0f;
    /// <summary>
    /// How on target does this turret need to be to consider firing.
    /// </summary>
    [Export(PropertyHint.Range, "0.0, 3.1416, 0.01")]
    public float EffectiveAngle = 0.05f;

    /// <summary>
    /// Used by player.
    /// </summary>
    [Export]
    public bool AutoFire = true;

    public void Verify()
    {

    }
}


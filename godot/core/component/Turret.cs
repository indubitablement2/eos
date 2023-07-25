using Godot;
using System;

[Tool]
public partial class Turret : Sprite2D
{
    [Export]
    public float FiringArc;

    [Export]
    public float Range;

    [Export]
    public bool IsBuiltin;

    bool HasAmmo;
    public int MaxAmmo;
    public float AmmoReplenishDelay;
    public int AmmoReplenishAmount;

    public float FiringDelay;


    Entity Parent;



    public bool IsFiringArcFull()
    {
        return FiringArc > Mathf.Tau;
    }

    public virtual void TryFire()
    {

    }

    public override void _Draw()
    {
        if (IsFiringArcFull())
        {
            DrawArc(
                Vector2.Zero,
                Range,
                0.0f,
                Mathf.Tau,
                32,
                new Color(1.0f, 0.0f, 0.0f, 0.5f),
                1.0f,
                true
            );
        }
        else
        {
            DrawLine(
                Vector2.Zero,
                new Vector2(Range - 1.5f, 0.0f).Rotated(FiringArc * -0.5f),
                new Color(1.0f, 0.0f, 0.0f, 0.5f),
                1.0f,
                true
            );
            DrawLine(
                Vector2.Zero,
                new Vector2(Range - 1.5f, 0.0f).Rotated(FiringArc * 0.5f),
                new Color(1.0f, 0.0f, 0.0f, 0.5f),
                1.0f,
                true
            );

            DrawArc(
                Vector2.Zero,
                Range,
                FiringArc * -0.5f,
                FiringArc * 0.5f,
                32,
                new Color(1.0f, 0.0f, 0.0f, 0.5f),
                1.0f,
                true
            );
        }


    }

    // Called when the node enters the scene tree for the first time.
    public override void _Ready()
    {
    }

    // Called every frame. 'delta' is the elapsed time since the previous frame.
    public override void _Process(double delta)
    {

    }
}

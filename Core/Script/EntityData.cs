using Godot;
using System;
using System.Collections.Generic;


public partial class EntityData : Sprite2D
{
    // TODO: Maybe create a xml from data builder using tool script
    // that way we can merge them
    // TODO: How to do custom script?
    public static Dictionary<string, EntityData> Datas = new();

    [Export]
    public float Mass = 1.0f;


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
    public Entity.WishLinearVeloctyEnum WishLinearVeloctyType;
    [Export]
    public Vector2 WishLinearVelocity = Vector2.Zero;

    [Export]
    public Entity.WishAngularVelocityEnum WishAngularVelocityType;
    [Export]
    public float WishAngularVelocity = 0.0f;


}
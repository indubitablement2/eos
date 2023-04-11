using Godot;
using System;

public class EntityData
{
    public PackedScene EntityScene;
    public Texture2D Icon;

    public float LinearAcceleration;
    public float AngularAcceleration;
    public float MaxLinearVelocity;
    public float MaxAngularVelocity;

    public float Readiness;
    public float HullHp;
    public float ArmorHp;

    public EntityData(Resource entityDataResource)
    {
        EntityScene = (PackedScene)entityDataResource.Get("EntityScenePath");

        RigidBody2D entityScene = EntityScene.Instantiate<RigidBody2D>();

        // RigidBody setup.
        entityScene.CenterOfMassMode = RigidBody2D.CenterOfMassModeEnum.Custom;
        entityScene.CanSleep = false;

        // Get the icon.
        Sprite2D sprite = entityScene.GetNode<Sprite2D>("Sprite2D");
        Icon = sprite.Texture;

        // TODO: Set sprite material.

        EntityScene.Pack(entityScene);
        entityScene.Free();

        LinearAcceleration = (float)entityDataResource.Get("LinearAcceleration");
        AngularAcceleration = (float)entityDataResource.Get("AngularAcceleration");
        MaxLinearVelocity = (float)entityDataResource.Get("MaxLinearVelocity");
        MaxAngularVelocity = (float)entityDataResource.Get("MaxAngularVelocity");
        Readiness = (float)entityDataResource.Get("Readiness");
        HullHp = (float)entityDataResource.Get("HullHp");
        ArmorHp = (float)entityDataResource.Get("ArmorHp");
    }
}
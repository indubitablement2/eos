using Godot;
using System;

public partial class ShipData : Resource
{
    [Export]
    public PackedScene EntityShipScene;

    [Export]
    public String DisplayName = "";

    [Export]
    public Texture2D Icon = Constants.ErrorTexture;

    public float LinearAcceleration;
    public float AngularAcceleration;
    public float MaxLinearVelocity;
    public float MaxAngularVelocity;

    public float Readiness;
    public float HullHp;
    public float ArmorHp;

    public void FetchBaseStats()
    {
        if (EntityShipScene == null)
        {
            return;
        }

        EntityShip entityShip = EntityShipScene.Instantiate<EntityShip>();

        LinearAcceleration = entityShip.LinearAcceleration;
        AngularAcceleration = entityShip.AngularAcceleration;
        MaxLinearVelocity = entityShip.MaxLinearVelocity;
        MaxAngularVelocity = entityShip.MaxAngularVelocity;

        Readiness = entityShip.Readiness;
        HullHp = entityShip.HullHp;
        ArmorHp = entityShip.ArmorHp;

        entityShip.Free();
    }

    public EntityShip InstantiateEntity()
    {
        return EntityShipScene.Instantiate<EntityShip>();
    }
}
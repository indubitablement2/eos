using Godot;
using System;
using System.Diagnostics;

public class Ship
{
    Fleet Fleet;

    ShipData ShipData;

    float Readiness;
    float HullHp;
    float ArmorHp;

    // <summary>
    // null if the ship is not spawned in battle.
    // </summary>
    EntityShip Entity;

    public Ship(ShipData shipData, Fleet fleet)
    {
        ShipData = shipData;
        Fleet = fleet;

        Readiness = shipData.Readiness;
        HullHp = shipData.HullHp;
        ArmorHp = shipData.ArmorHp;
    }

    public Ship(ShipData shipData, Fleet fleet, float readiness, float hullHp, float armorHp)
    {
        ShipData = shipData;
        Fleet = fleet;

        Readiness = readiness;
        HullHp = hullHp;
        ArmorHp = armorHp;
    }

    /// <summary>
    /// Return null if the ship is not ready.
    /// </summary>
    public EntityShip SpawnEntity()
    {
        if (Entity != null)
        {
            GD.PushWarning("Ship is not ready to be spawned.");
            return null;
        }

        Entity = ShipData.InstantiateEntity();

        Entity.OwnerClientId = Fleet.OwnerClient.ClientId;
        Entity.Readiness = Readiness;
        Entity.HullHp = HullHp;
        Entity.TreeExiting += OnEntityExiting;

        Battlescape.ShipAdded(Entity);

        return Entity;
    }

    void OnEntityExiting()
    {
        // TODO: Check if the ship is destroyed.

        Readiness = Math.Min(Entity.Readiness, Readiness * 0.8f);
        HullHp = Math.Min(Entity.HullHp, HullHp);
        ArmorHp = Math.Min(Entity.GetAverageArmorHp(), ArmorHp);

        Entity = null;
    }
}
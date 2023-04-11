using Godot;
using System;

public class Ship
{
    public enum ShipState
    {
        Ready,
        Battlescape,
    }
    public ShipState State;

    public Fleet Fleet;

    public ShipData ShipData;

    public float Readiness;
    public float HullHp;
    public float ArmorHp;

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
    public EntityShip TrySpawnEntity(BattlescapeSimulation battlescapeSimulation)
    {
        if (State != ShipState.Ready)
        {
            return null;
        }

        State = ShipState.Battlescape;

        EntityShip scene = ShipData.EntityData.EntityScene.Instantiate<EntityShip>();
        battlescapeSimulation.AddChild(scene);
        scene.Initialize(this, battlescapeSimulation);

        return scene;
    }
}
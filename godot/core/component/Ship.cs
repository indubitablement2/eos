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

    public EntityData Data;
    public EntityStats Stats;

    Ship(Fleet fleet)
    {
        Fleet = fleet;
        Fleet.Ships.Add(this);
    }

    /// <summary>
    /// Return null if the ship is not ready.
    /// </summary>
    public EntityShip TrySpawnEntity()
    {
        if (State != ShipState.Ready)
            return null;

        State = ShipState.Battlescape;
        return new EntityShip(this);
    }
}
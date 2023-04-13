using Godot;
using System;
using System.Collections.Generic;

public class Fleet
{
    public Int64 FleetId;

    /// <summary>
    /// null if the fleet is not owned by a client.
    /// </summary>
    public Client OwnerClient;

    public List<Ship> Ships = new List<Ship>();

    public bool InBattle;

    public Fleet(Int64 fleetId, Client ownerClient = null)
    {
        FleetId = fleetId;
        OwnerClient = ownerClient;
    }

    public Ship CreateShip(ShipData shipData)
    {
        Ship ship = new Ship(shipData, this);
        Ships.Add(ship);
        return ship;
    }

    public Ship CreateShip(ShipData shipData, float readiness, float hullHp, float armorHp)
    {
        Ship ship = new Ship(shipData, this, readiness, hullHp, armorHp);
        Ships.Add(ship);
        return ship;
    }

    public void SetInBattle(bool inBattle)
    {
        InBattle = inBattle;
    }
}
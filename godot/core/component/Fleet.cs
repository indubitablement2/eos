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

    public List<Ship> Ships;

    /// <summary>
    /// null if the fleet is not in a battlescape.
    /// </summary>
    public Battlescape Battlescape;

    public Fleet(Int64 fleetId, List<Ship> ships, Client ownerClient = null)
    {
        FleetId = fleetId;
        OwnerClient = ownerClient;
        Ships = ships;
        Battlescape = null;
    }

    public bool TryEnterBattlescape(Battlescape battlescape)
    {
        if (Battlescape != null)
        {
            return false;
        }

        Battlescape = battlescape;
        Battlescape.AddFleet(this);
        return true;
    }

    public bool TryExitBattlescape()
    {
        if (Battlescape == null)
        {
            return false;
        }

        Battlescape.RemoveFleet(this);
        Battlescape = null;
        return true;
    }
}
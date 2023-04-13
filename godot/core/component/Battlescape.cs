using Godot;
using System;
using System.Collections.Generic;
using System.Diagnostics;

public partial class Battlescape : Node2D
{
    Int64 Tick;

    List<Fleet> Fleets = new List<Fleet>();

    Client Host;

    // TODO: Initial state
    public static Battlescape CreateLocal()
    {
        Battlescape battlescape = new Battlescape();
        battlescape.Host = Main.LocalClient;
        return battlescape;
    }

    public void AddFleet(Fleet fleet)
    {
        Debug.Assert(!fleet.InBattle);

        fleet.SetInBattle(true);
        Fleets.Add(fleet);
    }

    public void AddShip(EntityShip entity)
    {
        // TODO: Position
        // TODO: Team

        AddChild(entity);
    }

    // public bool IsPaused()
    // {
    //     return ProcessMode == ProcessModeEnum.Disabled;
    // }

    // public void SetPaused(bool paused)
    // {
    //     if (paused)
    //     {
    //         ProcessMode = ProcessModeEnum.Disabled;
    //     }
    //     else
    //     {
    //         ProcessMode = ProcessModeEnum.Inherit;
    //     }
    // }

    // public override void _PhysicsProcess(double delta)
    // {

    // }
}
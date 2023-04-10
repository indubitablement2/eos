using Godot;
using System;
using System.Collections.Generic;

public class Battlescape
{
    public Int64 BattlescapeId;

    public Int64 StartTick;

    public List<Fleet> Fleets;

    /// <summary>
    /// not null if the battlescape is simulated by player(s), but not necessarily localy.
    /// </summary>
    public Client Host;
    static PackedScene _battlescapeScene = ResourceLoader.Load<PackedScene>(
        "res://core/scene/BattlescapeSimulation.scn"
    );
    /// <summary>
    /// not null when the battlescape is simulated by the local player.
    /// </summary>
    public BattlescapeSimulation Simulation;

    public Battlescape(Int64 battlescapeId)
    {
        BattlescapeId = battlescapeId;
        StartTick = Metascape.Tick;
        Fleets = new List<Fleet>();
    }

    public bool IsSimulated()
    {
        return Host != null;
    }

    public bool StartSimulation(Client host)
    {
        if (IsSimulated())
        {
            return false;
        }

        Host = host;

        if (Host.ClientId == Metascape.LocalClientId)
        {
            CreateLocalSimulation();
        }

        return true;
    }

    // TODO: Initial state
    void CreateLocalSimulation()
    {
        Simulation = _battlescapeScene.Instantiate<BattlescapeSimulation>();
        Simulation.Battlescape = this;
        Metascape.Root.AddChild(Simulation);
    }

    public void AddFleet(Fleet fleet)
    {
        Fleets.Add(fleet);
    }

    public void RemoveFleet(Fleet fleet)
    {
        Fleets.Remove(fleet);
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
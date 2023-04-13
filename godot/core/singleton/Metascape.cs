using Godot;
using System;
using System.Collections.Generic;

public partial class Metascape : Node2D
{
    public Int64 Tick;

    public Client Host;

    public Dictionary<Int64, Client> Clients = new Dictionary<Int64, Client>();

    Int64 _nextFleetId;
    public Dictionary<Int64, Fleet> Fleets = new Dictionary<Int64, Fleet>();

    public static Metascape CreateLocal()
    {
        Metascape metascape = new Metascape();
        metascape.Host = Main.LocalClient;
        metascape.Clients.Add(Main.LocalClient.ClientId, Main.LocalClient);
        return metascape;
    }

    public static Metascape ConnectMultiplayer()
    {
        // TODO: Connect to server
        return null;
    }

    public Fleet CreateFleet(Client ownerClient = null)
    {
        Fleet fleet = new Fleet(_nextFleetId, ownerClient);
        Fleets.Add(_nextFleetId, fleet);
        _nextFleetId += 1;
        return fleet;
    }

    public bool IsPaused()
    {
        return ProcessMode == ProcessModeEnum.Disabled;
    }

    public void SetPaused(bool paused)
    {
        if (paused)
        {
            ProcessMode = ProcessModeEnum.Disabled;
        }
        else
        {
            ProcessMode = ProcessModeEnum.Inherit;
        }
    }

    public override void _Ready()
    {
    }

    public override void _PhysicsProcess(double delta)
    {
        Tick += 1;
    }
}
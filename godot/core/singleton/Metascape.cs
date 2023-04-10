using Godot;
using System;
using System.Collections.Generic;

public partial class Metascape : Node2D
{
    public static Int64 Tick;
    public static Metascape Root;

    static Int64 _nextClientId;
    public static Dictionary<Int64, Client> Clients;
    public static Int64 LocalClientId;
    public static Client CreateClient()
    {
        Client client = new Client(_nextClientId);
        Clients.Add(_nextClientId, client);
        _nextClientId += 1;
        return client;
    }

    static Int64 _nextFleetId;
    public static Dictionary<Int64, Fleet> Fleets;
    public static Fleet CreateFleet(Client ownerClient = null)
    {
        // TODO: Add ships
        Fleet fleet = new Fleet(_nextFleetId, new List<Ship>(), ownerClient);
        Fleets.Add(_nextFleetId, fleet);
        _nextFleetId += 1;
        return fleet;
    }

    static Int64 _nextBattlescapeId;
    public static Dictionary<Int64, Battlescape> Battlescapes;
    public static Battlescape CreateBattlescape()
    {
        Battlescape battlescape = new Battlescape(_nextBattlescapeId);
        Battlescapes.Add(_nextBattlescapeId, battlescape);
        _nextBattlescapeId += 1;
        return battlescape;
    }

    public static void Reset()
    {
        Tick = 0;

        _nextClientId = 0;
        Clients = new Dictionary<Int64, Client>();
        LocalClientId = 0;

        _nextBattlescapeId = 0;
        Battlescapes = new Dictionary<Int64, Battlescape>();

        _nextFleetId = 0;
        Fleets = new Dictionary<Int64, Fleet>();

        foreach (Node node in Root.GetChildren())
        {
            node.QueueFree();
        }

        SetPaused(true);
    }

    public static bool IsPaused()
    {
        return Root.ProcessMode == ProcessModeEnum.Disabled;
    }

    public static void SetPaused(bool paused)
    {
        if (paused)
        {
            Root.ProcessMode = ProcessModeEnum.Disabled;
        }
        else
        {
            Root.ProcessMode = ProcessModeEnum.Inherit;
        }
    }

    public override void _Ready()
    {
        Root = this;
        Reset();
    }

    public override void _PhysicsProcess(double delta)
    {
        Tick += 1;
    }
}
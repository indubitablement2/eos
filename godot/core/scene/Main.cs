using Godot;
using System;
using System.Collections.Generic;

public partial class Main : Node
{
    static Main Root;

    public static Metascape Metascape;

    // TODO: Use steamworks
    // public static Dictionary<Int64, Client> Connections = new Dictionary<Int64, Client>();

    public static Client LocalClient = Client.Default();

    public static void ShowMetascape()
    {
        Battlescape.Root.Hide();
        Metascape?.Show();
    }

    public static void ShowBattlescape()
    {
        Metascape?.Hide();
        Battlescape.Root.Show();
    }

    public static void AddMetascape(Metascape metascape)
    {
        if (Metascape != null)
        {
            GD.PushError("Metascape already exists.");
            return;
        }

        Metascape = metascape;
        Root.AddChild(Metascape);

        Metascape.TreeExiting += OnMetascapeTreeExiting;
    }

    public override void _Ready()
    {
        Root = this;

        Settings.Load();
        Data.LoadData();
        Data.PrintData();

        // TODO: Find client. Use steamworks if available.
        // LocalClient = new Client(-1);

        // TODO: Just for testing. Remove this.
        AddMetascape(Metascape.CreateLocal());

        Fleet fleet = Metascape.CreateFleet(LocalClient);
        fleet.JoinBattlescape();
        for (int i = 0; i < 4; i++)
        {
            fleet.CreateShip(Data.ShipDatas["res://core/data/Fallback/FallbackShipData.tres"])
            .SpawnEntity();
        }

        ShowBattlescape();
        Metascape.SetPaused(false);
    }

    static void OnMetascapeTreeExiting()
    {
        Metascape = null;
    }
}

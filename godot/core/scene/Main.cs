using Godot;
using System;
using System.Collections.Generic;

public partial class Main : Control
{
    public static Main Root;

    public static TextureRect FocussedBattlescapeRender;
    public static BattlescapeSimulation FocussedBattlescapeSimulation;

    public static VBoxContainer UnfocussedBattlescapeRenderContainer;

    public override void _Ready()
    {
        Root = this;
        FocussedBattlescapeRender = GetNode<TextureRect>("FocussedBattlescapeRender");
        UnfocussedBattlescapeRenderContainer = GetNode<VBoxContainer>(
            "MarginContainer/UnfocussedBattlescapeRenderContainer"
        );

        // TODO: Just for testing. Remove this.
        Data.LoadData();

        Client localClient = Metascape.CreateClient();
        Metascape.LocalClientId = localClient.ClientId;

        Fleet fleet = Metascape.CreateFleet(localClient);

        Battlescape battlescape = Metascape.CreateBattlescape();

        fleet.TryEnterBattlescape(battlescape);
        battlescape.StartSimulation(localClient);

        Metascape.SetPaused(false);
    }

    public override void _Process(double delta)
    {
    }
}

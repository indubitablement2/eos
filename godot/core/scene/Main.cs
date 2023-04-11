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

        ShipData shipData = Data.ShipDatas["res://base/ship/test/TestShipData.tres"];
        fleet.AddShip(
            shipData,
            shipData.EntityData.Readiness,
            shipData.EntityData.HullHp,
            shipData.EntityData.ArmorHp
        );

        Battlescape battlescape = Metascape.CreateBattlescape();

        fleet.TryEnterBattlescape(battlescape);
        battlescape.StartSimulation(localClient);

        fleet.Ships[0].TrySpawnEntity(battlescape.Simulation);

        Metascape.SetPaused(false);
    }

    public override void _Process(double delta)
    {
    }
}

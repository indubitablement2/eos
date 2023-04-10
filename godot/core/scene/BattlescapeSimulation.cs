using Godot;
using System;
using System.Collections.Generic;

public partial class BattlescapeSimulation : SubViewport
{
    public Battlescape Battlescape;

    public bool IsHost;

    public TextureRect UnfocussedRender;

    public override void _Ready()
    {
        UnfocussedRender = new TextureRect();
        UnfocussedRender.Texture = GetTexture();
        Main.UnfocussedBattlescapeRenderContainer.AddChild(UnfocussedRender);
        SetFocussed(false);

        if (Battlescape.Host.IsLocal())
        {

            IsHost = true;
        }
        else
        {
            // TODO: 

            IsHost = false;
        }
    }

    public override void _PhysicsProcess(double delta)
    {

    }

    public override void _Process(double delta)
    {
        if (Main.FocussedBattlescapeSimulation == this)
        {
            if (Size != GetWindow().Size)
            {
                Size = GetWindow().Size;
            }
        }
        else
        {
            if (Size != Settings.UnfocusedBattlescapeRenderSize)
            {
                Size = Settings.UnfocusedBattlescapeRenderSize;
            }
        }
    }

    public void SetFocussed(bool focussed)
    {
        if (focussed)
        {
            if (Main.FocussedBattlescapeSimulation != null)
            {
                Main.FocussedBattlescapeSimulation.SetFocussed(false);
            }

            UnfocussedRender.Hide();

            Main.FocussedBattlescapeRender.Texture = GetTexture();
            Main.FocussedBattlescapeSimulation = this;
        }
        else
        {
            if (Main.FocussedBattlescapeSimulation == this)
            {
                Main.FocussedBattlescapeRender.Texture = null;
                Main.FocussedBattlescapeSimulation = null;
            }

            UnfocussedRender.Show();
        }
    }
}

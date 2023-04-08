using Godot;
using System;

public partial class Battlescape : Node2D 
{
    public static Int64 Tick = 0;
    public static Battlescape Root = null;

    public static void Reset()
    {
        Tick = 0;
    }

    public static void Kill()
    {
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
        GD.Print(Root);
    }

    public override void _PhysicsProcess(double delta)
    {
        Tick += 1;
    }
}
using Godot;
using System;
using System.Collections.Generic;
using System.Diagnostics;

public class BattlescapeSettings
{
    public float BoundRadius = 2000.0f;
}

public partial class Battlescape : Node2D
{
    public static Node2D Root;

    static Int64 Tick;

    static List<Fleet> Fleets;

    static Client Host;

    public static float BoundRadius;
    public static float BoundRadiusSquared;
    public static float SafeBoundRadiusSquared;
    public static void SetBoundRadius(float radius)
    {
        BoundRadius = radius;
        BoundRadiusSquared = radius * radius;
        SafeBoundRadiusSquared = (radius - 500.0f) * (radius - 500.0f);
        OnBoundRadiusChanged?.Invoke();
    }

    public static event Action OnBoundRadiusChanged;
    public static event Action OnReset;

    public static void Reset(BattlescapeSettings settings = null)
    {
        if (settings == null)
        {
            settings = new BattlescapeSettings();
        }

        foreach (Node child in Root.GetChildren())
        {
            child.QueueFree();
        }

        Tick = 0;
        Fleets = new List<Fleet>();
        Host = Main.LocalClient;
        SetBoundRadius(settings.BoundRadius);

        OnReset?.Invoke();
    }

    public static void FleetAdded(Fleet fleet)
    {
        Fleets.Add(fleet);
    }

    public static void ShipAdded(EntityShip entity)
    {
        // TODO: Position
        // TODO: Team

        Root.AddChild(entity);
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


    public override void _Ready()
    {
        Root = this;
        Reset();
    }

    public override void _PhysicsProcess(double delta)
    {
        Tick++;
    }
}
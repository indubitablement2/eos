using Godot;
using System;

public partial class ActionInputs : Node2D
{
    static Node2D Root;
    static Viewport Viewport;

    static StringName ActionUp = "up";
    static StringName ActionDown = "down";
    static StringName ActionLeft = "left";
    static StringName ActionRight = "right";
    static StringName ActionStrafeLeft = "strafe_left";
    static StringName ActionStrafeRight = "strafe_right";
    static StringName ActionCancelLinearVelocity = "cancel_linear_velocity";
    static StringName ActionFaceCursor = "face_cursor";

    public static bool CancelLinearVelocity;
    public static bool FaceCursor;

    static bool GlobalMousePositionCached;
    static Vector2 GlobalMousePositionCache;
    public static Vector2 GetCachedGlobalMousePosition()
    {
        if (!GlobalMousePositionCached)
        {
            GlobalMousePositionCache = Root.GetGlobalMousePosition();
            GlobalMousePositionCached = true;
        }

        return GlobalMousePositionCache;
    }

    static bool VerticalDirectionCached;
    static float VerticalDirectionCache;
    /// <summary>
    /// up is positive, down is negative.
    /// </summary>
    public static float GetCachedVecticalDirection()
    {
        if (!VerticalDirectionCached)
        {
            VerticalDirectionCache = Input.GetActionStrength(ActionUp) - Input.GetActionStrength(ActionDown);
            VerticalDirectionCached = true;
        }

        return VerticalDirectionCache;
    }

    static bool HorizontalDirectionCached;
    static float HorizontalDirectionCache;
    /// <summary>
    /// right is positive, left is negative.
    /// </summary>
    public static float GetCachedHorizontalDirection()
    {
        if (!HorizontalDirectionCached)
        {
            HorizontalDirectionCache = Input.GetActionStrength(ActionRight) - Input.GetActionStrength(ActionLeft);
            HorizontalDirectionCached = true;
        }

        return HorizontalDirectionCache;
    }

    static bool StrafeDirectionCached;
    static float StrafeDirectionCache;
    /// <summary>
    /// right is positive, left is negative.
    /// </summary>
    public static float GetCachedStrafeDirection()
    {
        if (!StrafeDirectionCached)
        {
            StrafeDirectionCache = Input.GetActionStrength(ActionStrafeRight) - Input.GetActionStrength(ActionStrafeLeft);
            StrafeDirectionCached = true;
        }

        return StrafeDirectionCache;
    }

    public override void _Ready()
    {
        Root = this;
        Viewport = GetViewport();
    }

    public override void _UnhandledInput(InputEvent @event)
    {
        if (@event.IsAction(ActionCancelLinearVelocity))
        {
            CancelLinearVelocity = @event.IsPressed();
        }
        else if (@event.IsAction(ActionFaceCursor))
        {
            FaceCursor = @event.IsPressed();
        }

        // This is the only place where we handle input events.
        Viewport.SetInputAsHandled();
    }

    public override void _Process(double delta)
    {
        GlobalMousePositionCached = false;
        VerticalDirectionCached = false;
        HorizontalDirectionCached = false;
        StrafeDirectionCached = false;
    }
}
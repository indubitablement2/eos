using System;
using Godot;

public static class Settings
{
    public static Vector2I UnfocusedBattlescapeRenderSize = new Vector2I(128, 128);
    public static event Action OnUnfocusedBattlescapeRenderSizeChanged;
    public static void SetUnfocusedBattlescapeRenderSize(Vector2I value)
    {
        UnfocusedBattlescapeRenderSize = value;
        OnUnfocusedBattlescapeRenderSizeChanged?.Invoke();
    }

    public static void Load()
    {
        // TODO: Load the settings from disk.
    }

    public static void Save()
    {
        // TODO: Save the settings to disk.
    }
}